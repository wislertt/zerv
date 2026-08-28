import os
import re
import shutil
import zipfile
from pathlib import Path
from typing import Annotated

import typer
from bake import Context, command, console
from bakelib import GitHubActionsTools, PythonSpace, RustSpace, params
from bakelib.publisher.crates import CratesPublisher
from bakelib.publisher.pypi import PyPIPublisher as _PyPIPublisher
from bakelib.space.lib import BaseLibSpace

from tests.python.utils import symlink_zerv_to_venv_bin


class PyPIPublisher(_PyPIPublisher):
    """Custom PyPI publisher for zerv that uses maturin instead of uv build."""

    _target: str | None = None

    def _build_for_publish(self, ctx: Context) -> None:
        cmd = "maturin build --release --strip --out dist/"
        env: dict[str, str] | None = None

        if self._target:
            cmd += f" --target {self._target}"

        if self._target and "-linux-" in self._target:
            compatibility = "musllinux_1_2" if self._target.endswith("-musl") else "manylinux_2_17"
            cmd += f" --zig --compatibility {compatibility}"
            # zig comes from the venv's `python-zig` (maturin[zig]); a system maturin
            # would not see it, so make sure the venv's bin dir wins
            venv_bin = Path(".venv/bin").resolve()
            env = {"PATH": f"{venv_bin}{os.pathsep}{os.environ['PATH']}"}

        ctx.run(cmd, env=env)

        if self._target and "-linux-" in self._target:
            self._check_wheel_glibc()

        # One designated leg also ships the sdist so non-matrix platforms have a fallback
        if self._target == "x86_64-unknown-linux-gnu":
            ctx.run("maturin sdist --out dist/")

    def _check_wheel_glibc(self) -> None:
        """Fail the publish if the built binary breaks the wheel's platform tag.

        maturin tags the wheel from the --compatibility flag we pass, not from the
        binary's actual symbol references; this is the enforcement of that claim.
        """
        wheels = list(Path("dist").glob("*.whl"))
        if len(wheels) != 1:
            console.error(f"Expected exactly one wheel in dist/, found: {[w.name for w in wheels]}")
            raise typer.Exit(1)

        with zipfile.ZipFile(wheels[0]) as zf:
            # bindings = "bin" places the binary in <pkg>.data/scripts/ (find_bin)
            bins = [name for name in zf.namelist() if name.endswith(".data/scripts/zerv")]
            if len(bins) != 1:
                console.error(f"Expected one bundled binary in {wheels[0].name}, found: {bins}")
                raise typer.Exit(1)
            binary = zf.read(bins[0])

        text = binary.decode("ascii", errors="ignore")
        versions = {
            tuple(int(part) for part in match.split("."))
            for match in re.findall(r"GLIBC_(\d+(?:\.\d+)*)", text)
        }

        if self._target and self._target.endswith("-musl"):
            if versions:
                console.error(
                    f"musl wheel {wheels[0].name} must be static but references glibc: "
                    f"{sorted(versions)}"
                )
                raise typer.Exit(1)
            console.success(f"{wheels[0].name}: static, no glibc references")
        else:
            highest = max(versions, default=(0,))
            if highest > (2, 17):
                console.error(
                    f"{wheels[0].name} claims manylinux_2_17 but binary requires "
                    f"glibc {'.'.join(map(str, highest))}"
                )
                raise typer.Exit(1)
            console.success(
                f"{wheels[0].name}: max glibc reference "
                f"{'.'.join(map(str, highest)) if versions else 'none'}"
            )


class MyBakebook(RustSpace, PythonSpace, GitHubActionsTools, BaseLibSpace):
    zerv_test_native_git: bool = False
    zerv_test_docker: bool = True
    zerv_force_rust_log_off: bool = False
    _target: str | None = None

    def _get_mise_tools(self) -> set[str]:
        mise_tools = super()._get_mise_tools()
        mise_tools.add("npm:mintlify")
        return mise_tools

    def get_publish_registries(self) -> set[str]:
        return set(PyPIPublisher.valid_registries) | set(CratesPublisher.valid_registries)

    def get_publisher(self, registry: str) -> PyPIPublisher | CratesPublisher:
        """Return the appropriate publisher, using custom PyPIPublisher for maturin builds."""
        if registry in PyPIPublisher.valid_registries:
            publisher = PyPIPublisher(registry)
            publisher._target = self._target
            return publisher
        if registry in CratesPublisher.valid_registries:
            return CratesPublisher(registry)

        valid = (*PyPIPublisher.valid_registries, *CratesPublisher.valid_registries)
        console.error(f"Invalid registry: {registry!r}. Expected one of {valid}.")
        raise typer.Exit(1)

    def _update_config(self, **kwargs: bool | None) -> None:
        for key, value in kwargs.items():
            if value is not None:
                setattr(self, key, value)

    @command()
    def test_rust(
        self,
        *,
        zerv_test_native_git: bool | None = None,
        zerv_test_docker: bool | None = None,
        zerv_force_rust_log_off: bool | None = None,
    ):
        self._update_config(
            zerv_test_native_git=zerv_test_native_git,
            zerv_test_docker=zerv_test_docker,
            zerv_force_rust_log_off=zerv_force_rust_log_off,
        )

        env: dict[str, str] = {}
        env["ZERV_TEST_NATIVE_GIT"] = str(self.zerv_test_native_git).lower()
        env["ZERV_TEST_DOCKER"] = str(self.zerv_test_docker).lower()
        env["ZERV_FORCE_RUST_LOG_OFF"] = str(self.zerv_force_rust_log_off).lower()
        env["RUST_BACKTRACE"] = "1"
        env["RUST_LOG"] = "cargo_tarpaulin=off"

        self.ctx.run(
            "cargo tarpaulin "
            "--features test-utils "
            "--out Xml --out Html --out Lcov "
            "--output-dir coverage "
            "--include-tests "
            "--exclude-files 'src/main.rs' "
            "--exclude-files '**/tests/**' "
            "--exclude-files 'src/test_utils/git/native.rs' "
            "-- --quiet",
            env=env,
            shell=True,
        )

    @command()
    def test_python(
        self,
        build: Annotated[
            bool, typer.Option("--build", "-b", help="Build before running tests")
        ] = False,
    ):
        if build:
            self.ctx.run("maturin develop")
            if not self.ctx.dry_run:
                symlink_zerv_to_venv_bin()
        tests_path = "tests/python"
        coverage_path = "python/zerv"
        self._test(tests_paths=tests_path, coverage_path=coverage_path)

    def test(
        self,
        *,
        zerv_test_native_git: bool | None = None,
        zerv_test_docker: bool | None = None,
        zerv_force_rust_log_off: bool | None = None,
    ) -> None:
        self._update_config(
            zerv_test_native_git=zerv_test_native_git,
            zerv_test_docker=zerv_test_docker,
            zerv_force_rust_log_off=zerv_force_rust_log_off,
        )

        self.test_rust()
        self.test_python(build=True)

    @command()
    def docs(self):
        self.ctx.run("mintlify dev", cwd=Path("docs"))

    @command()
    def docs_check(self):
        self.ctx.run("mintlify broken-links", cwd=Path("docs"))

    @command()
    def open_coverage(self):
        self.ctx.run("open coverage/tarpaulin-report.html")

    @command()
    def extract_mermaid_svgs(self):
        self.ctx.run("./scripts/extract_mermaid_from_markers.sh")

    @command()
    def publish(
        self,
        *,
        registry: Annotated[
            str,
            typer.Option(help="Publish registry (test-pypi, pypi, or crates)"),
        ] = "test-pypi",
        token: params.PublishTokenOption = None,
        version: params.PublishVersionOption = None,
        target: Annotated[
            str | None,
            typer.Option(
                help="Rust target triple (e.g., aarch64-apple-darwin, x86_64-pc-windows-msvc)"
            ),
        ] = None,
    ):
        self._target = target
        return super().publish(registry=registry, token=token, version=version)

    def _get_version(self) -> str:
        return self._get_consistent_version((RustSpace, PythonSpace))

    def _pre_publish_setup(self) -> None:
        """Custom pre-publish setup for zerv - handles both Rust and Python."""
        # zerv uses itself for versioning in _version_bump_context, so build and symlink it first
        self.ctx.run("maturin develop")
        if not self.ctx.dry_run:
            symlink_zerv_to_venv_bin()

        # Call BOTH publishers' setup (zerv is multi-lang)
        CratesPublisher._pre_publish_setup(self.ctx)  # removes target/package
        PyPIPublisher._pre_publish_setup(self.ctx)  # removes dist

        # maturin
        for p in Path("python").glob("*.data"):
            if p.is_dir():
                shutil.rmtree(p)


bakebook = MyBakebook()


@bakebook.command()
def uvx_install_zerv_test():
    bakebook.ctx.run(
        "uv tool install zerv-version "
        "--index-url https://test.pypi.org/simple/ "
        "--extra-index-url https://pypi.org/simple "
        "--prerelease allow "
        "--reinstall "
        "--index-strategy unsafe-best-match"
    )
