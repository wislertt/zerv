from bake import Context, command
from bakelib import PythonLibSpace


class MyBakebook(PythonLibSpace):
    zerv_test_native_git: bool = False
    zerv_test_docker: bool = True
    zerv_force_rust_log_off: bool = False

    def _update_config(self, **kwargs: bool | None) -> None:
        for key, value in kwargs.items():
            if value is not None:
                setattr(self, key, value)

    def update(self, ctx: Context):
        super().update(ctx)
        ctx.run("rustup update")
        ctx.run("cargo update")

    def lint(self, ctx: Context) -> None:
        super().lint(ctx=ctx)
        ctx.run("cargo +nightly check --tests")
        ctx.run("cargo +nightly fmt -- --check || (cargo +nightly fmt && exit 1)")
        ctx.run("cargo +nightly clippy --all-targets --all-features -- -D warnings")

    # TODO: update this to bakefile
    def _test(
        self,
        ctx: Context,
        *,
        tests_paths: str | list[str],
        verbose: bool = False,
        coverage_report: bool = True,
        coverage_path: str = "src",
    ) -> None:
        paths = tests_paths if isinstance(tests_paths, str) else " ".join(tests_paths)

        cmd = f"uv run pytest {paths}"

        if coverage_report:
            cmd += f" --cov={coverage_path} --cov-report=html --cov-report=term-missing --cov-report=xml"

        if verbose:
            cmd += " -s -v"

        ctx.run(cmd)

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


bakebook = MyBakebook()
