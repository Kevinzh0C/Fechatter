// 🚨 Fechatter 紧急消息菜单修复脚本
// 在浏览器控制台中执行此脚本

console.log('🚨 启动 Fechatter 紧急消息菜单修复...');

function emergencyMessageMenuFix() {
    let fixes = [];
    
    // 1. 检查并修复DiscordMessageItem工具栏
    const discordMessages = document.querySelectorAll('.discord-message');
    if (discordMessages.length > 0) {
        fixes.push(`✅ 找到 ${discordMessages.length} 个Discord消息`);
        
        discordMessages.forEach((msg, index) => {
            // 强制显示工具栏
            const toolbar = msg.querySelector('.message-toolbar');
            if (toolbar) {
                toolbar.style.display = 'flex';
                toolbar.style.visibility = 'visible';
                toolbar.style.opacity = '1';
                toolbar.style.position = 'absolute';
                toolbar.style.right = '8px';
                toolbar.style.top = '8px';
                toolbar.style.zIndex = '1000';
                toolbar.style.background = 'rgba(255, 255, 255, 0.9)';
                toolbar.style.borderRadius = '6px';
                toolbar.style.padding = '4px 8px';
                toolbar.style.boxShadow = '0 2px 8px rgba(0,0,0,0.1)';
            }
            
            // 触发鼠标悬停状态
            if (msg.__vue__ && msg.__vue__.handleMouseEnter) {
                msg.__vue__.handleMouseEnter();
            }
            
            // 设置showToolbar状态
            if (msg.__vue__) {
                msg.__vue__.showToolbar = true;
                msg.__vue__.isMouseOver = true;
            }
        });
        
        fixes.push(`🔧 修复了 ${discordMessages.length} 个消息工具栏`);
    } else {
        fixes.push('❌ 未找到Discord消息组件');
    }
    
    // 2. 创建紧急CSS修复
    const style = document.createElement('style');
    style.id = 'emergency-menu-fix';
    style.textContent = `
        /* 🚨 紧急修复：强制显示消息菜单 */
        .discord-message .message-toolbar {
            display: flex !important;
            visibility: visible !important;
            opacity: 1 !important;
            position: absolute !important;
            right: 8px !important;
            top: 8px !important;
            z-index: 1000 !important;
            background: rgba(255, 255, 255, 0.95) !important;
            border: 1px solid #e5e7eb !important;
            border-radius: 6px !important;
            padding: 4px 8px !important;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1) !important;
            gap: 4px !important;
        }
        
        .discord-message:hover .message-toolbar {
            display: flex !important;
        }
        
        .toolbar-btn {
            display: flex !important;
            align-items: center !important;
            justify-content: center !important;
            width: 32px !important;
            height: 32px !important;
            border: none !important;
            background: transparent !important;
            cursor: pointer !important;
            border-radius: 4px !important;
            color: #6b7280 !important;
        }
        
        .toolbar-btn:hover {
            background: rgba(0, 0, 0, 0.05) !important;
            color: #374151 !important;
        }
        
        /* 特别修复翻译按钮 */
        .toolbar-btn[title*="ranslat"],
        .toolbar-btn[aria-label*="ranslat"] {
            background: #fff3cd !important;
            color: #856404 !important;
            border: 1px solid #ffeaa7 !important;
        }
        
        /* 强制显示所有按钮图标 */
        .toolbar-btn svg,
        .toolbar-btn [name="language"] {
            display: block !important;
            width: 16px !important;
            height: 16px !important;
        }
    `;
    
    // 移除旧样式
    const oldStyle = document.getElementById('emergency-menu-fix');
    if (oldStyle) oldStyle.remove();
    
    document.head.appendChild(style);
    fixes.push('✅ 应用了紧急CSS修复');
    
    // 3. 强制所有消息元素可见
    document.querySelectorAll('.message-item, .discord-message').forEach(msg => {
        msg.style.display = 'block';
        msg.style.visibility = 'visible';
        msg.style.opacity = '1';
        msg.style.minHeight = '60px';
        msg.style.position = 'relative';
    });
    
    // 4. 强制所有文本内容可见
    document.querySelectorAll('.message-text, .message-content').forEach(text => {
        text.style.display = 'block';
        text.style.visibility = 'visible';
        text.style.opacity = '1';
        text.style.minHeight = '1em';
        text.style.color = '#374151';
    });
    
    fixes.push('✅ 强制显示所有消息内容');
    
    // 5. 检查翻译按钮
    const translationButtons = document.querySelectorAll('[title*="ranslat"], [aria-label*="ranslat"]');
    if (translationButtons.length > 0) {
        translationButtons.forEach(btn => {
            btn.style.background = '#fff3cd';
            btn.style.border = '2px solid #856404';
            btn.style.borderRadius = '4px';
        });
        fixes.push(`✅ 高亮了 ${translationButtons.length} 个翻译按钮`);
    } else {
        fixes.push('⚠️ 未找到翻译按钮，可能需要检查组件状态');
    }
    
    // 输出修复结果
    console.log('🎉 紧急修复完成:');
    fixes.forEach(fix => console.log(fix));
    
    return {
        success: true,
        fixes: fixes,
        messageCount: discordMessages.length,
        translationButtonCount: translationButtons.length
    };
}

// 执行修复
const result = emergencyMessageMenuFix();

// 提供额外的调试命令
window.debugMessageMenus = function() {
    console.log('🔍 调试消息菜单状态:');
    
    document.querySelectorAll('.discord-message').forEach((msg, index) => {
        const toolbar = msg.querySelector('.message-toolbar');
        const vue = msg.__vue__;
        
        console.log(`Message ${index}:`, {
            hasToolbar: !!toolbar,
            toolbarVisible: toolbar ? getComputedStyle(toolbar).display !== 'none' : false,
            vueState: vue ? {
                showToolbar: vue.showToolbar,
                showTranslationOptions: vue.showTranslationOptions,
                isMouseOver: vue.isMouseOver
            } : 'No Vue instance',
            element: msg
        });
    });
};

window.forceMenusVisible = function() {
    console.log('🔧 强制显示所有菜单...');
    emergencyMessageMenuFix();
};

console.log('💡 可用调试命令:');
console.log('  debugMessageMenus() - 调试菜单状态');
console.log('  forceMenusVisible() - 强制显示菜单');
console.log('');
console.log('🎯 如果问题仍然存在，请检查:');
console.log('  1. 是否正确加载了DiscordMessageItem组件');
console.log('  2. Vue实例的状态管理是否正常');
console.log('  3. 鼠标事件监听器是否正确绑定');

