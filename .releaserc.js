module.exports = {
    branches: ["main"],
    plugins: [
        [
            "@semantic-release/commit-analyzer",
            {
                preset: "angular",
                releaseRules: [
                    // Pre-alpha configuration: all commit types trigger patch releases
                    // TODO: Change feat to "minor" when project reaches stable state
                    { type: "feat", release: "patch" },
                    { type: "fix", release: "patch" },
                    { type: "chore", release: "patch" },
                    { type: "docs", release: "patch" },
                    { type: "style", release: "patch" },
                    { type: "refactor", release: "patch" },
                    { type: "perf", release: "patch" },
                    { type: "test", release: "patch" },
                    { type: "build", release: "patch" },
                    { type: "ci", release: "patch" },
                ],
            },
        ],
        "@semantic-release/release-notes-generator",
        "@semantic-release/github",
    ],
}
