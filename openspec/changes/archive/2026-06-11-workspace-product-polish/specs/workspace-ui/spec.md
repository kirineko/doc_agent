## MODIFIED Requirements

### Requirement: 三栏工作区布局
系统 SHALL 提供三栏布局：左侧为项目 / 会话 / 模型配置，中间为会话与结果，右侧为工具调用链与项目文件浏览（上下分栏）。

#### Scenario: 三栏同时可见
- **WHEN** 用户打开一个项目的会话
- **THEN** 界面同时呈现左侧导航与配置、中间会话区、右侧工具调用链与文件浏览两个区域

### Requirement: 右侧工具调用链可视化
系统 SHALL 在右侧栏上半区以简洁美观的方式展示工具调用链，每个调用呈现名称、参数、状态与结果（含耗时）；下半区留给项目文件浏览，二者共享右侧栏宽度且各自可纵向滚动。

#### Scenario: 展示工具调用进展
- **WHEN** Agent 发起并完成一个工具调用
- **THEN** 右侧栏上半区出现对应卡片，状态从「执行中」更新为「完成 / 失败」，并显示结果摘要与耗时

## ADDED Requirements

### Requirement: 应用品牌标识
系统 SHALL 使用定制 Logo 替换 Tauri 默认图标，并在顶栏标题旁展示 Logo 图形；窗口标题文案保持「Doc Agent」。

#### Scenario: 顶栏展示 Logo
- **WHEN** 用户打开应用主窗口
- **THEN** 顶栏左侧显示定制 Logo 与「Doc Agent」文字，而非仅纯文字或 Tauri 默认标识

#### Scenario: 安装包与窗口使用定制图标
- **WHEN** 用户安装或运行打包后的应用
- **THEN** 快捷方式、任务栏与 macOS Dock 显示定制图标，而非 Tauri 默认图标

### Requirement: 安装目录无空格
系统 SHALL 将打包产物的默认安装目录名设为 `DocAgent`（无空格）；用户可见窗口标题不受此约束。

#### Scenario: Windows 默认安装路径
- **WHEN** 用户在 Windows 上执行默认安装
- **THEN** 默认目标目录为 `DocAgent` 而非含空格的 `Doc Agent`
