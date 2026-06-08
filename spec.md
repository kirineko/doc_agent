1. 支持Windows和macos的桌面应用，体积要尽量小，UI要美观
2. 核心功能是Agent Loop和工具调用
3. 强化Word、Excel、PPT的读写与生成功能，主要用户群体是办公人员
4. 模型支持DeepSeek V4 Flash：https://api-docs.deepseek.com/zh-cn/guides/thinking_mode
5. 模型支持Kimi k2.6：https://platform.kimi.com/docs/guide/use-kimi-k2-thinking-model#%E4%BD%BF%E7%94%A8%E9%A1%BB%E7%9F%A5
7. 用户选择目录，以该目录作为项目，作为操作的基本单位，Agent只能在目录范围内进行操作；针对每一个项目，可以建立多个会话，每个会话作为独立上下文，会话之间暂不共享记忆。会话历史和过程中的工具调用要进行持久化。
8. 左侧侧边栏支持查看项目和会话列表，支持选择和配置model，如DeepSeek V4 Flash、DeepSeek V4 Pro、Kimi K2.6，支持配置thinking开关，支持选择思考强度（high、max（kimi只有开关没有思考强度））
9. 中间显示会话和结果，要做好markdown渲染
10. 右侧显示工具调用链，要求简洁美观