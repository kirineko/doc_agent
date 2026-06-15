## 1. 字体资源与构建

- [x] 1.1 build.rs 下载 Noto Subset SC（Regular/Bold × Sans/Serif）
- [x] 1.2 gitignore `src-tauri/fonts/`；tauri.conf.json 添加 resources

## 2. 编译链

- [x] 2.1 `compile.rs` 配置 `include_dirs` 与资源路径解析
- [x] 2.2 `lib.rs` 启动时 `configure_font_dir`

## 3. 模板

- [x] 3.1 平台字体栈 + 更新 `fonts.typ` Noto 族名与 `covers`

## 4. 验证

- [x] 4.1 测试：exam-zh 编译无字体警告
- [x] 4.2 `cargo test` 通过
