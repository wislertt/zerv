import shutil
from pathlib import Path
from typing import Annotated

import typer
import zerv
from bake import command, console
from bakelib import PythonSpace, RustSpace
from bakelib.publisher import Publisher
from bakelib.publisher.crates import CratesPublisher
from bakelib.publisher.pypi import PyPIPublisher as _PyPIPublisher
from bakelib.space.lib import BaseLibSpace
from bakelib.space.params import publish_token_option, publish_version_option

from tests.python.utils import symlink_zerv_to_venv_bin


class PyPIPublisher(_PyPIPublisher):
    """Custom PyPI publisher for zerv that uses maturin instead of uv build."""

    _target: str | None = None

    def _build_for_publish(self) -> None:
        cmd = "maturin build --release --strip --out dist/"

        if self._target:
            cmd += f" --target {self._target}"

        self.ctx.run(cmd)


class MyBakebook(RustSpace, PythonSpace, BaseLibSpace):
    zerv_test_native_git: bool = False
    zerv_test_docker: bool = True
    zerv_force_rust_log_off: bool = False
    _target: str | None = None

    def get_publisher(self, registry: str) -> PyPIPublisher | CratesPublisher:
        """Return the appropriate publisher, using custom PyPIPublisher for maturin builds."""
        if registry in PyPIPublisher.valid_registries:
            publisher = PyPIPublisher(self.ctx, registry)
            publisher._target = self._target
            return publisher
        if registry in CratesPublisher.valid_registries:
            return CratesPublisher(self.ctx, registry)

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
    def gen_docs(self):
        self.ctx.run("cargo xtask generate-docs")

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
        token: publish_token_option = None,
        version: publish_version_option = None,
        target: Annotated[
            str | None,
            typer.Option(
                help="Rust target triple (e.g., aarch64-apple-darwin, x86_64-pc-windows-msvc)"
            ),
        ] = None,
    ):
        self._target = target
        return super().publish(registry=registry, token=token, version=version)

    @property
    def _version(self) -> str:
        cargo_raw = RustSpace._version.fget(self)
        pyproject_raw = PythonSpace._version.fget(self)

        pyproject_semver = zerv.render(version=pyproject_raw, output_format="semver")
        cargo_semver = zerv.render(version=cargo_raw, output_format="semver")

        if pyproject_semver != cargo_semver:
            raise ValueError(
                f"Version mismatch: pyproject.toml={pyproject_raw} ({pyproject_semver}), "
                f"Cargo.toml={cargo_raw} ({cargo_semver})"
            )

        return cargo_raw

    @_version.setter
    def _version(self, value: str) -> None:
        RustSpace._version.fset(self, value)
        PythonSpace._version.fset(self, value)

    def _pre_publish_setup(self, publisher: Publisher) -> None:
        """Custom pre-publish setup for zerv - handles both Rust and Python."""
        _ = publisher

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
