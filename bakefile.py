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


def _wheel_build_command(target: str | None) -> tuple[str, dict[str, str] | None]:
    cmd = "maturin build --release --strip --out dist/"
    env: dict[str, str] | None = None

    if target:
        cmd += f" --target {target}"

    if target and "-linux-" in target:
        compatibility = "musllinux_1_2" if target.endswith("-musl") else "manylinux_2_17"
        cmd += f" --zig --compatibility {compatibility}"
        # zig comes from the venv's python-zig (maturin[zig]); a system maturin would not see it
        venv_bin = Path(".venv/bin").resolve()
        env = {"PATH": f"{venv_bin}{os.pathsep}{os.environ['PATH']}"}

    return cmd, env


def _build_wheel(ctx: Context, target: str | None) -> None:
    cmd, env = _wheel_build_command(target)
    ctx.run(cmd, env=env)

    if target and "-linux-" in target:
        _check_wheel_glibc(target)


def _extract_wheel_binary() -> Path:
    wheels = list(Path("dist").glob("*.whl"))
    if len(wheels) != 1:
        console.error(f"Expected exactly one wheel in dist/, found: {[w.name for w in wheels]}")
        raise typer.Exit(1)

    with zipfile.ZipFile(wheels[0]) as zf:
        bins = [
            name
            for name in zf.namelist()
            if name.endswith((".data/scripts/zerv", ".data/scripts/zerv.exe"))
        ]
        if len(bins) != 1:
            console.error(f"Expected one bundled binary in {wheels[0].name}, found: {bins}")
            raise typer.Exit(1)
        binary_path = Path("dist") / Path(bins[0]).name
        binary_path.write_bytes(zf.read(bins[0]))

    binary_path.chmod(0o755)
    console.success(f"Extracted {binary_path} from {wheels[0].name}")
    return binary_path


def _check_wheel_glibc(target: str | None) -> None:
    """maturin tags the wheel from --compatibility, not the binary's symbols; enforce the claim."""
    wheels = list(Path("dist").glob("*.whl"))
    if len(wheels) != 1:
        console.error(f"Expected exactly one wheel in dist/, found: {[w.name for w in wheels]}")
        raise typer.Exit(1)

    with zipfile.ZipFile(wheels[0]) as zf:
        # bindings = "bin" places the binary in <pkg>.data/scripts/ (find_bin)
        bins = [
            name
            for name in zf.namelist()
            if name.endswith((".data/scripts/zerv", ".data/scripts/zerv.exe"))
        ]
        if len(bins) != 1:
            console.error(f"Expected one bundled binary in {wheels[0].name}, found: {bins}")
            raise typer.Exit(1)
        binary = zf.read(bins[0])

    text = binary.decode("ascii", errors="ignore")
    versions = {
        tuple(int(part) for part in match.split("."))
        for match in re.findall(r"GLIBC_(\d+(?:\.\d+)*)", text)
    }

    if target and target.endswith("-musl"):
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


class PyPIPublisher(_PyPIPublisher):
    """Custom PyPI publisher for zerv that uses maturin instead of uv build."""

    _target: str | None = None

    def _build_for_publish(self, ctx: Context) -> None:
        _build_wheel(ctx, self._target)

        # One designated leg also ships the sdist so non-matrix platforms have a fallback
        if self._target == "x86_64-unknown-linux-gnu":
            ctx.run("maturin sdist --out dist/")


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

    @command(help="Build a release binary for a target triple, extracted from the maturin wheel")
    def build(
        self,
        *,
        target: Annotated[
            str | None,
            typer.Option(
                help="Rust target triple (e.g., aarch64-apple-darwin, x86_64-pc-windows-msvc)"
            ),
        ] = None,
    ):
        shutil.rmtree("dist", ignore_errors=True)
        _build_wheel(self.ctx, target)
        return _extract_wheel_binary()

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
