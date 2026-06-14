# CX - 开发文档管理

CX 是一个通用的开发文档管理体系，用于规划、跟踪和记录软件开发的迭代过程。

## 核心理念

遵循 **设计 → 开发 → 验证 → 反思 → 改进** 的闭环迭代流程。

## 适用场景

- 独立跟踪多个项目的开发迭代
- 文档与代码分离，便于并行管理
- 团队知识沉淀与复盘

## 文档结构

```
docs/superpowers/
├── specs/          # 架构设计文档
├── plans/          # 开发计划与任务清单
├── reviews/        # 代码审查与验证记录
├── iterations/     # 迭代总结与经验教训
└── improvements/   # 优化改进计划
```

## 快速开始

1. **新建设计文档**：`docs/superpowers/specs/YYYY-MM-DD-<name>-design.md`
2. **制定开发计划**：`docs/superpowers/plans/YYYY-MM-DD-<name>.md`
3. **记录迭代总结**：`docs/superpowers/iterations/YYYY-MM-DD-<iteration>-summary.md`

## 文档命名规范

| 类型 | 格式 | 示例 |
|------|------|------|
| 设计文档 | `YYYY-MM-DD-<name>-design.md` | `2026-06-14-api-gateway-design.md` |
| 开发计划 | `YYYY-MM-DD-<name>.md` | `2026-06-14-api-gateway.md` |
| 验证记录 | `YYYY-MM-DD-<name>-review.md` | `2026-06-14-api-gateway-review.md` |
| 迭代总结 | `YYYY-MM-DD-<iteration>-summary.md` | `2026-06-14-sprint1-summary.md` |
| 改进计划 | `YYYY-MM-DD-<improvement>.md` | `2026-06-14-performance.md` |

## 使用指南

详见 `.claude/工作流程.md`

## 关联项目

本文档可与任意代码仓库配合使用，通过路径或链接建立关联。

示例：
- 文档仓库：`E:/work/cx`
- 代码仓库：`E:/work/GA2`
