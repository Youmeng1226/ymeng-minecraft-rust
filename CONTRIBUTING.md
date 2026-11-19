# 贡献指南

感谢你对本项目感兴趣！

## 开发流程

1. Fork 仓库并新建分支：`git checkout -b feature/your-feature`
2. 修改代码，并为新逻辑补上单元测试
3. 提交前在本地运行：

   ```bash
   cargo fmt --all
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

4. 提交 PR，说明改了什么、为什么改

## 代码约定

- `ymc-core` 不引入外部依赖。
- 公共 API 写简短的文档注释。
- 涉及性能的改动请附上前后对比数据。

## 适合新手的任务

在 [路线图](docs/ROADMAP.md) 中挑一个未完成的条目，开 Issue 认领即可。
