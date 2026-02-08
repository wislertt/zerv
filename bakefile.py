import subprocess
from typing import Annotated, get_args

import typer
from bake import command, console
from bakelib import PythonLibSpace, RustLibSpace
from bakelib.space.lib import BaseLibSpace, PublishResult
from bakelib.space.python_lib import PyPIRegistry
from bakelib.space.rust_lib import CratesRegistry


class MyBakebook(RustLibSpace, PythonLibSpace):
    zerv_test_native_git: bool = False
    zerv_test_docker: bool = True
    zerv_force_rust_log_off: bool = False
    __registry: CratesRegistry | PyPIRegistry | None = None

    def _build_for_publish(self):
        self.ctx.run("maturin build --release")

    def _handle_publish_result(self, publish_result: PublishResult) -> None:
        if self.ctx.dry_run:
            return

        elif publish_result.is_auth_failed:
            console.error("Authentication failed. Please check your publish token.")
            raise typer.Exit(1)

        elif publish_result.result is None:
            console.error("Publish result is empty (unexpected).")
            raise typer.Exit(1)

        elif publish_result.result.returncode == 0:
            if publish_result.is_dry_run:
                console.warning(
                    "This was a dry-run. To actually publish, "
                    "set the BAKE_PUBLISH_TOKEN environment variable"
                )
                return

            console.success("Publish succeeded!")
            return

        elif publish_result.result.returncode != 0:
            console.error(
                "Publish failed with unexpected error. "
                f"Return code: {publish_result.result.returncode}"
            )
            raise typer.Exit(1)

        console.error("Unexpected publish result state")
        raise typer.Exit(1)

    @property
    def _registry(self) -> CratesRegistry | PyPIRegistry:
        if self.__registry is None:
            raise RuntimeError("_registry not set")
        return self.__registry

    @_registry.setter
    def _registry(self, value: str):
        self.__registry = self._validate_registry(value)

    @property
    def _publish_impl(self) -> type[BaseLibSpace]:
        if self._registry in get_args(PyPIRegistry):
            return PythonLibSpace
        if self._registry in get_args(CratesRegistry):
            return RustLibSpace

        valid = (*get_args(PyPIRegistry), *get_args(CratesRegistry))
        console.error(f"Invalid registry: {self._registry!r}. Expected one of {valid}.")
        raise typer.Exit(1)

    def _update_config(self, **kwargs: bool | None) -> None:
        for key, value in kwargs.items():
            if value is not None:
                setattr(self, key, value)

    @command()
    def rust_test(
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
    def python_test(self, build: bool = False):
        if build:
            self.ctx.run("maturin develop")
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

        self.rust_test()
        self.python_test(build=True)

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
            typer.Option(help="Publish registry (testpypi, pypi, or crates)"),
        ] = "testpypi",
        token: Annotated[str | None, typer.Option(help="Publish token")] = None,
        version: Annotated[str | None, typer.Option(help="Version to publish")] = None,
    ):
        self._registry = registry
        return self._publish_impl.publish(
            self, registry=self._registry, token=token, version=version
        )

    def _validate_registry(self, registry: str) -> CratesRegistry | PyPIRegistry:  # type: ignore[invalid-method-override]
        if registry in get_args(PyPIRegistry):
            impl = PythonLibSpace
        elif registry in get_args(CratesRegistry):
            impl = RustLibSpace
        else:
            valid = (*get_args(PyPIRegistry), *get_args(CratesRegistry))
            console.error(f"Invalid registry: {registry!r}. Expected one of {valid}.")
            raise typer.Exit(1)

        valid_registry = impl._validate_registry(self, registry)

        return valid_registry

    @property
    def _version_schema(self) -> str | None:
        return self._publish_impl._version_schema.fget(self)

    @property
    def _version_output_format(self) -> str | None:
        return self._publish_impl._version_output_format.fget(self)

    def _get_publish_token_from_remote(self, registry: str) -> str | None:
        return self._publish_impl._get_publish_token_from_remote(self, registry)

    def _build_for_publish(self):
        return self._publish_impl._build_for_publish(self)

    def _publish_with_token(self, token: str | None, registry: str) -> PublishResult:
        return self._publish_impl._publish_with_token(self, token, registry)

    def _is_auth_failure(self, result: subprocess.CompletedProcess[str]) -> bool:
        return self._publish_impl._is_auth_failure(self, result)

    def _version_bump_context(self, version: str):
        return self._publish_impl._version_bump_context(self, version)

    def _pre_publish_cleanup(self):
        return self._publish_impl._pre_publish_cleanup(self)


bakebook = MyBakebook()
