# Skip-Stone (打水漂模拟器) — 场景搭建与配置指南

## 核心概念

gdextension 向 Godot 注册**新的节点类型**。这些自定义类型集成了原生节点的能力 + Rust 侧的行为逻辑——等同于"原生节点 + 脚本"的合体。

- 在编辑器中通过 **"添加子节点" → 搜索类名** 来创建自定义节点
- **不需要、也不能**再为它们附加 `.gd`/`.cs` 脚本
- 原生节点（`Camera2D`、`CollisionShape2D`、`StaticBody2D` 等）正常使用

---

## 前置条件

| 项 | 值 |
|----|-----|
| Godot | **4.7** |
| DLL | `res://skip_stone.dll`（已提供） |
| gdextension 配置 | `rust.gdextension` 已就绪 |
| 渲染后端 | D3D12（`project.godot` 已配置） |

> 如果编辑器里搜不到自定义节点：确认 DLL 在 `res://` 下，然后**重启 Godot**。gdextension 只在启动时加载。

---

## 自定义节点清单

| 类名 | 继承自 | 在编辑器中搜索 | 职责 |
|------|--------|---------------|------|
| `GameManager` | `Node2D` | `GameManager` | 状态机、瞄准输入、加速点生成、UI 构建、相机跟随 |
| `Ball` | `RigidBody2D` | `Ball` | 石子物理体、冻结/发射、距离追踪、地面碰撞、自绘外观 |
| `Water` | `Area2D` | `Water` | 打水漂/沉水判定、虚线动画 |
| `Aim` | `Area2D` | `Aim` | 瞄准线段可视化 |

---

## 完整节点树

```
Game             ← 自定义: GameManager (Change Type → 搜索 GameManager)
│
├─ Background    ← 原生: Node2D
│  ├─ FarLayer   ← 原生: Parallax2D (远层)
│  │  └─ HintFar ← 原生: Label ("拖拽那个球", 大字)
│  └─ NearLayer  ← 原生: Parallax2D (近层，与 FarLayer 是兄弟节点)
│     └─ HintNear ← 原生: Label ("松手即可发射", 小字)
│
├─ Ball          ← 自定义: Ball (Add Child → 搜索 Ball)
│  └─ CollisionShape2D ← 原生 (CircleShape2D, r=20)
│
├─ Water         ← 自定义: Water（虚线由 Rust 侧 draw() 绘制）
│  └─ CollisionShape2D ← 原生 (SegmentShape2D)
│
├─ Ground        ← 原生: StaticBody2D
│  ├─ CollisionShape2D ← 原生 (RectangleShape2D, 40000×200)
│  └─ Line2D     ← 原生 (黑色实线)
│
├─ Aim           ← 自定义: Aim
│  ├─ CollisionShape2D ← 原生 (CircleShape2D, r=90)
│  └─ Line2D     ← 原生 (代码控制显隐)
│
└─ Camera2D      ← 原生
```

> 距离标签 (`DistanceLabel`)、结算面板 (`ResultPanel`)、加速点由 `GameManager` 在运行时动态创建，不需要在场景中预置。

---

## 逐节点详细配置

### 1. `Game` — 自定义节点 `GameManager`

**创建**：场景默认根节点是 `Node2D` → 右键 → **Change Type** → 搜索 `GameManager`。

| 属性 | 值 |
|------|-----|
| 节点类型 | `GameManager` |

其余默认。`ready()` 中自动连接子节点信号、初始化 Ball 状态、构建 UI。

---

### 2. `Background` — 原生 `Node2D`

> **在此之前**：菜单 **Project → Project Settings → Rendering → Environment → Default Clear Color** 设为 **`#DDDDDD`**。这样游戏窗口默认底色就是浅灰，无需放置 ColorRect。Ball 填充色为 `#FFFFFF`（白），与背景形成清晰对比。

纯容器节点，两个兄弟 `Parallax2D` 作为直接子节点。

---

### 3. `FarLayer` — 原生 `Parallax2D`（远层，Background 的第 1 个子节点）

| 属性 | 值 |
|------|-----|
| `Motion → Scale` | `(0.1, 0.1)` | 慢跟随，远景感 |

#### 3a. `HintFar` — 原生 `Label`

| 属性 | 值 |
|------|-----|
| `Text` | `拖拽那个球` |
| `Horizontal Alignment` | `Center` |
| `Theme Overrides → Font Size` | `48` px |
| `Theme Overrides → Font Color` | `#000000` |
| 位置 | 居中偏上 |

---

### 4. `NearLayer` — 原生 `Parallax2D`（近层，Background 的第 2 个子节点，与 FarLayer 兄弟）

| 属性 | 值 |
|------|-----|
| `Motion → Scale` | `(0.3, 0.3)` | 比远层快 |

#### 4a. `HintNear` — 原生 `Label`

| 属性 | 值 |
|------|-----|
| `Text` | `松手即可发射` |
| `Horizontal Alignment` | `Center` |
| `Theme Overrides → Font Size` | `36` px |
| `Theme Overrides → Font Color` | `#000000` |
| 位置 | 居中偏下 |

---

### 5. `Ball` — 自定义节点 `Ball`

**创建**：右键 `Game` → **Add Child Node** → 搜索 `Ball`。

`Ball` 的外观由 Rust 侧 `draw()` 自动绘制：`#FFFFFF` 填充 + 黑色描边、半径 20px、圆心在本节点坐标系 `(0, 0)`。

| 属性 | 值 | 说明 |
|------|----|------|
| 节点类型 | `Ball` | 自定义，继承 RigidBody2D |
| `Collision → Layer` | **仅勾选第 1 位** | Ball 在碰撞层 1 |
| `Collision → Mask` | **仅勾选第 2 位** | 仅物理碰撞层 2（地面） |
| `Collision → Contact Monitor` | **✓** | 接收 body_entered 信号 |
| `Collision → Max Contacts` | `1` | |
| `Physics → Freeze` | **✓** | 发射前物理冻结 |
| `Physics → Freeze Mode` | **Kinematic** | 冻结时仍可通过代码移动位置 |
| `Physics → Gravity Scale` | `1.0` | |
| `Linear → Damp` | `0` | |
| `Angular → Damp` | `0` | |

#### 5a. `Ball/CollisionShape2D`

| 属性 | 值 |
|------|-----|
| `Shape` | **`CircleShape2D`** |
| `Shape → Radius` | **`20`** px |
| Position | `(0, 0)`（本地坐标，圆心与 Ball 原点重合） |

---

### 6. `Water` — 自定义节点 `Water`

**创建**：右键 `Game` → Add Child Node → 搜索 `Water`。

虚线外观由 Rust 侧 `draw()` 直接绘制（黑色、1.5px 宽、8px 虚线段 + 8px 间隔、向右滚动），**不需要**任何子节点。

| 属性 | 值 | 说明 |
|------|----|------|
| 节点类型 | `Water` | 自定义，继承 Area2D |
| `Collision → Monitoring` | **✓** | |
| `Collision → Mask` | **仅勾选第 1 位** | 检测 Ball（层 1）进入 |
| `Transform → Position → Y` | `600` | 水面高度 |

#### 6a. `Water/CollisionShape2D`

| 属性 | 值 |
|------|-----|
| `Shape` | **`SegmentShape2D`** |
| `Shape → a` | `(-20000, 0)` |
| `Shape → b` | `(20000, 0)` |

---

### 7. `Ground` — 原生 `StaticBody2D`

| 属性 | 值 | 说明 |
|------|----|------|
| 节点类型 | `StaticBody2D` | |
| `Collision → Layer` | **仅勾选第 2 位** | 地面在碰撞层 2 |
| **Groups** | **添加条目 `ground`** | Ball 通过 `is_in_group("ground")` 识别 |
| `Transform → Position → Y` | `720` | 窗口底部 |

#### 7a. `Ground/CollisionShape2D`

为防止高速下落的球穿透地面，使用有纵深的矩形替代线段：

| 属性 | 值 |
|------|-----|
| `Shape` | **`RectangleShape2D`** |
| `Shape → Size` | `(40000, 200)` | 宽无限，高 200px 纵深 |
| `Position` | `(0, -100)` | 矩形顶部对齐地面线 y=720（父节点 Y 已设 720，Shape 向上偏移一半高度使顶边为 0） |

> 不设 position 也行——球会在矩形内部停下来，略低于 y=720，配合撞击痕迹视觉效果自然。

#### 7b. `Ground/Line2D`

| 属性 | 值 |
|------|-----|
| `Width` | `2` |
| `Default Color` | `#000000` |
| `Points` | `[(-20000, 0), (20000, 0)]` |

---

### 8. `Aim` — 自定义节点 `Aim`

**创建**：右键 `Game` → Add Child Node → 搜索 `Aim`。

| 属性 | 值 | 说明 |
|------|----|------|
| 节点类型 | `Aim` | 自定义，继承 Area2D |
| `Collision → Monitoring` | **✗ OFF** | 纯视觉，无碰撞 |

#### 8a. `Aim/CollisionShape2D`

| 属性 | 值 |
|------|-----|
| `Shape` | **`CircleShape2D`** |
| `Shape → Radius` | **`90`** px |
| `Position` | `(320, 480)` | 圆心 = Ball 初始位置 |

#### 8b. `Aim/Line2D`

| 属性 | 值 |
|------|-----|
| `Width` | `2` |
| `Default Color` | `#000000` |
| `Visible` | **✗** | 代码在拖拽时显示 |
| `Points` | 留空（代码动态设置） |

---

### 9. `Camera2D` — 原生

| 属性 | 值 |
|------|-----|
| `Current` | **✓ ON** |
| `Anchor Mode` | `Drag Center` |
| `Position` | `(640, 360)` |

> 飞行后平滑跟随 Ball，硬性保持 Ball 在世界坐标 `[256,1024] × [144,576]` 范围内（一个小于 1280×720 的矩形框）。

---

## 碰撞层总览

**Project Settings → Layer Names → 2D Physics**（命名可选）：

| 层编号 | 归属 |
|--------|------|
| Layer 1 | Ball（在层 1）、Water / Accelerator（mask 层 1） |
| Layer 2 | Ground（在层 2）、Ball（mask 层 2） |
| Layer 3 | 动态加速点（在层 3） |

**交互关系**：

| 从 | 到 | 方式 | 结果 |
|----|-----|------|------|
| Ball（物理体, mask=2） | Ground（层 2） | `body_entered` | Ball 停止 |
| Water（区域, mask=1） | Ball（层 1） | `body_entered` | 打水漂或沉水 |
| Accelerator（区域, mask=1） | Ball（层 1） | `body_entered` | 速度变化 |

---

## 游戏流程

```
[Aiming]   鼠标在 Ball 25px 内按下 → 拖拽（Ball 限制在 90px 瞄准圆内）
             松手：< 15px → Ball 弹回原位
                   ≥ 15px → 球沿拖拽反方向发射
                    v = 0.2 × drag_px  m/s  (= drag_px × 20  px/s)
                    │
                    ▼
[Flying]   Ball 飞行中
              ├─ 触 Water：入射角 ≤ 20° 且 v ≥ 4 m/s → 打水漂 (vx'=0.8vx, vy'=-0.8vy)
              │           否则 → 沉水 (gravity×0.2, vx'=0.5vx)
              ├─ 触加速点：vx'=1.1vx, vy'=1.1vy(向上) 或 -0.5vy(向下)
              ├─ 每累积 2000 px (20 m) 水平距离 → 右侧 (1280, Y随机) 生成新加速点
              └─ 触 Ground → 速度归零
                    │
                    ▼
[Grounded]  1 秒等待
                    │
                    ▼
[GameOver]  结算面板（本次距离 / 历史最高 / [重新开始]）
             点击按钮或 Enter → Aiming
```

---

## 速度公式说明

根据设计文档：`y = 0.2x`，其中 `x` 为拖拽长度（px），`y` 为初速度（m/s）。

代码中以 px/s 为单位（1 m = 100 px），因此：

```
v(px/s) = 0.2 × drag_px × 100 = drag_px × 20
```

对应常量 `DRAG_SCALE = 20.0`。

**示例**：
- 拖拽 15 px → 3 m/s = 300 px/s（恰好触发发射的阈值）
- 拖拽 90 px（最大）→ 18 m/s = 1800 px/s

---

## Rust 代码中的关键常量

在源码中修改后需重新 `cargo build`：

**`game_manager.rs`**：
```rust
AIM_RADIUS = 90.0            // 瞄准圆半径 (px)
LAUNCH_THRESHOLD = 15.0      // 触发发射的最小拖拽距离 (px)
DRAG_SCALE = 20.0            // v = drag × DRAG_SCALE (px/s)
BALL_SPAWN = (320, 480)      // Ball 初始位置
ACCEL_SPAWN_INTERVAL = 2000  // 加速点生成间距 (px, 即 20 m)
CAM_LOW_X / CAM_HIGH_X 等   // 相机边界
RESULT_DELAY_SEC = 1.0       // 碰地后结算延迟 (秒)
```

**`water.rs`**：
```rust
SKIP_ANGLE_DEG = 20.0        // 打水漂最大入射角
SKIP_SPEED_MS = 4.0          // 打水漂最低速度阈值 (m/s)
```

**`ball.rs`**：Ball 填充色 `#FFFFFF`、半径 20px 在 `draw()` 方法中硬编码。

---

## 构建

```bash
cd skip-stone/rust

cargo build              # 开发构建
cargo build --release    # 发布构建

# 复制 DLL（工作区 target 目录在上级）
cp ../target/debug/skip_stone.dll ../godot/skip_stone.dll
# 或
cp ../target/release/skip_stone.dll ../godot/skip_stone.dll
```

`rust.gdextension` 已开启 `reloadable = true`。修改代码 → `cargo build` → 复制 DLL → Godot 中 **Project → Reload Current Project**。
