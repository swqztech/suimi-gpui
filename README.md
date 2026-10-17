# suimi-gpui

[随密](https://github.com/swqztech/suimi) 的 **GPUI Fast** 实现：与 `suimi`（eframe）功能相同，使用 [gpui-kit](https://github.com/longbridge/gpui-kit) + `gpui-fast` 后端。

## 构建

```bash
cd suimi-gpui
cargo build --release
target\release\suimi-gpui.exe
```

## 说明

- 依赖 `gpui-kit` 的 `gpui-fast` feature，GPU 加速原生 UI
- 密码 / 用户名 / 网名、强度估计、复制、一次六个等与 egui 版一致
