- list git tags from current commit

❯ git tag --merged HEAD --sort=-committerdate
v1.0.2-rc.1.post.3
v1.1.0
v1.0.2-rc.1.post.2
v1.0.2-rc.1.post.1
v1.0.1
v1.0.0

- for loop one by one. select only parsable tag. find its commit

❯ git rev-list -n 1 v1.0.2-rc.1.post.3
2cdf46bfcb9d3a52c2d6a98006eabe42152212f5

- find all tags from that commit

❯ git tag --points-at 2cdf46bfcb9d3a52c2d6a98006eabe42152212f5
v1.0.2-rc.1.post.3
v1.1.0

- select the one with highest tag
