// 🚨 在浏览器控制台运行此代码立即修复消息可见性

console.log('🚀 开始立即修复消息可见性...');

// 强制移除所有高度限制
const fixStyle = document.createElement('style');
fixStyle.textContent = `
/* 🚨 紧急修复：移除所有容器高度限制 */
.simple-message-list {
    height: auto !important;
    max-height: none !important;
    overflow: visible !important;
}

.messages-wrapper {
    height: auto !important;
    min-height: auto !important;
    max-height: none !important;
    overflow: visible !important;
}

/* 🚨 强制所有消息可见 */
[data-message-id] {
    display: block !important;
    visibility: visible !important;
    opacity: 1 !important;
    position: relative !important;
    background: #f8fafc !important;
    border: 1px solid #e2e8f0 !important;
    margin: 8px 0 !important;
    padding: 12px !important;
    min-height: 60px !important;
}

[data-message-id] .message-text {
    display: block !important;
    visibility: visible !important;
    opacity: 1 !important;
    color: #1f2937 !important;
    font-size: 14px !important;
    background: rgba(59, 130, 246, 0.05) !important;
    padding: 4px !important;
    border-radius: 3px !important;
}
`;

document.head.appendChild(fixStyle);

// 检查结果
setTimeout(() => {
    const messages = document.querySelectorAll('[data-message-id]');
    const visible = Array.from(messages).filter(el => {
        const rect = el.getBoundingClientRect();
        return rect.height > 0;
    }).length;
    
    console.log(`✅ 修复完成！${visible}/${messages.length} 消息现在可见`);
    
    // 滚动到顶部查看所有消息
    const container = document.querySelector('.simple-message-list');
    if (container) {
        container.scrollTop = 0;
        console.log('📜 已滚动到顶部，现在可以看到所有消息');
    }
}, 1000);
