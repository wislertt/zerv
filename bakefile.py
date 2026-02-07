from typing import Annotated, Literal, get_args

import typer
from bake import Context, command
from bakelib import PythonLibSpace, RustLibSpace
from bakelib.space.python_lib import PyPIRegistry

CratesRegistry = Literal["crates"]


class MyBakebook(RustLibSpace, PythonLibSpace):
    zerv_test_native_git: bool = False
    zerv_test_docker: bool = True
    zerv_force_rust_log_off: bool = False

    def _update_config(self, **kwargs: bool | None) -> None:
        for key, value in kwargs.items():
            if value is not None:
                setattr(self, key, value)

    @command()
    def rust_test(
        self,
        ctx: Context,
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

        ctx.run(
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
    def python_test(self, ctx: Context, build: bool = False):
        if build:
            ctx.run("maturin develop")
        tests_path = "tests/python"
        coverage_path = "python/zerv"
        self._test(ctx, tests_paths=tests_path, coverage_path=coverage_path)

    def test(
        self,
        ctx: Context,
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

        self.rust_test(ctx)
        self.python_test(ctx, build=True)

    @command()
    def gen_docs(self, ctx: Context):
        ctx.run("cargo xtask generate-docs")

    @command()
    def open_coverage(self, ctx: Context):
        ctx.run("open coverage/tarpaulin-report.html")

    @command()
    def extract_mermaid_svgs(self, ctx: Context):
        ctx.run("./scripts/extract_mermaid_from_markers.sh")

    def publish(
        self,
        ctx: Context,
        *,
        registry: Annotated[
            str, typer.Option(help="Publish registry (testpypi, pypi, or crates)")
        ] = "testpypi",
        token: Annotated[str | None, typer.Option(help="Publish token")] = None,
        version: Annotated[str | None, typer.Option(help="Version to publish")] = None,
    ):
        if registry in get_args(PyPIRegistry):
            return PythonLibSpace.publish(
                self, ctx=ctx, registry=registry, token=token, version=version
            )
        return RustLibSpace.publish(self, ctx=ctx, registry=registry, token=token, version=version)


bakebook = MyBakebook()
