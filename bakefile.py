from bake import Context
from bakelib import PythonLibSpace


class MyBakebook(PythonLibSpace):
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

    def test(self, ctx: Context) -> None:
        tests_path = "tests/python"
        coverage_path = "python/zerv"
        self._test(ctx, tests_paths=tests_path, coverage_path=coverage_path)


bakebook = MyBakebook()
