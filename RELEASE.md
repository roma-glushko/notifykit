# Release

In order to release a new version of `notifykit`, follow these steps:

- bump the version in `pyproject.toml` and `Cargo.toml`. Merge the update into `main`.
- create a new tag with the version number, e.g. `0.0.9-alpha.1` and push it e.g.

```bash
git tag 0.0.9-alpha.1
git push --tag
```

This should start a new Github Action pipeline that will build the package and upload it to [PyPi](https://pypi.org/project/notifykit/#history).