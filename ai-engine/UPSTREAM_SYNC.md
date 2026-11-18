# ai-engine 上游同步

- 上游 https://github.com/umaai-project/umaai-matsuri
- commit c47e41da6c6eff5941f6f237d0a9817726b4a6b7
- 最近提交 c47e41d chore: 接入引擎 P0/P1 修复并重录基线；双智档换参定卡；帧还原补红点
- 引擎仓 https://github.com/umaai-project/umaai-next（submodule 钉住）
- commit 40a639d3a1878fe78c7982c9081dc29045fac321
- 最近提交 d5a75dc merge local changes -  P0/P1 修复（种子派生/事件状态位归属/固有20上限/智力回体闪彩过滤/分配防御）并重录 L4 基线
- 说明：手机端 App 是 Kotlin+WebView JS，不能直接运行本 Rust 引擎；此目录用于对照规则与移植。
