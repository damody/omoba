// 背景模糊佔位符
// 完整的實現將使用幀緩衝區讀回+高斯模糊
// 目前，這是一個呈現半透明覆蓋層的存根

pub struct BackdropBlurState {
    // 為未來基於 FBO 的模糊實現保留
}

impl BackdropBlurState {
    pub fn new() -> Self {
        Self {}
    }
}
