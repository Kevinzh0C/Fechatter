import { ref, nextTick } from 'vue';

export function useCodeBlockWrapper(textareaRef) {
  const showLanguageSelector = ref(false);
  const languageSelectorPosition = ref({ top: 0, left: 0 });
  const selectedRange = ref({ start: 0, end: 0 });
  
  // Popular programming languages
  const languages = [
    { code: 'javascript', name: 'JavaScript', aliases: ['js'] },
    { code: 'typescript', name: 'TypeScript', aliases: ['ts'] },
    { code: 'python', name: 'Python', aliases: ['py'] },
    { code: 'rust', name: 'Rust', aliases: ['rs'] },
    { code: 'go', name: 'Go', aliases: ['golang'] },
    { code: 'java', name: 'Java', aliases: [] },
    { code: 'cpp', name: 'C++', aliases: ['c++'] },
    { code: 'c', name: 'C', aliases: [] },
    { code: 'csharp', name: 'C#', aliases: ['cs'] },
    { code: 'php', name: 'PHP', aliases: [] },
    { code: 'ruby', name: 'Ruby', aliases: ['rb'] },
    { code: 'swift', name: 'Swift', aliases: [] },
    { code: 'kotlin', name: 'Kotlin', aliases: ['kt'] },
    { code: 'scala', name: 'Scala', aliases: [] },
    { code: 'sql', name: 'SQL', aliases: [] },
    { code: 'html', name: 'HTML', aliases: [] },
    { code: 'css', name: 'CSS', aliases: [] },
    { code: 'scss', name: 'SCSS', aliases: ['sass'] },
    { code: 'json', name: 'JSON', aliases: [] },
    { code: 'yaml', name: 'YAML', aliases: ['yml'] },
    { code: 'xml', name: 'XML', aliases: [] },
    { code: 'markdown', name: 'Markdown', aliases: ['md'] },
    { code: 'bash', name: 'Bash', aliases: ['sh', 'shell'] },
    { code: 'powershell', name: 'PowerShell', aliases: ['ps1'] },
    { code: 'dockerfile', name: 'Dockerfile', aliases: ['docker'] },
    { code: 'makefile', name: 'Makefile', aliases: ['make'] },
    { code: 'vue', name: 'Vue', aliases: [] },
    { code: 'jsx', name: 'JSX', aliases: [] },
    { code: 'tsx', name: 'TSX', aliases: [] },
    { code: 'graphql', name: 'GraphQL', aliases: ['gql'] },
    { code: 'lua', name: 'Lua', aliases: [] },
    { code: 'perl', name: 'Perl', aliases: ['pl'] },
    { code: 'r', name: 'R', aliases: [] },
    { code: 'dart', name: 'Dart', aliases: [] },
    { code: 'elixir', name: 'Elixir', aliases: ['ex'] },
    { code: 'erlang', name: 'Erlang', aliases: ['erl'] },
    { code: 'haskell', name: 'Haskell', aliases: ['hs'] },
    { code: 'ocaml', name: 'OCaml', aliases: ['ml'] },
    { code: 'clojure', name: 'Clojure', aliases: ['clj'] },
    { code: 'lisp', name: 'Lisp', aliases: [] },
    { code: 'scheme', name: 'Scheme', aliases: ['scm'] },
    { code: 'fsharp', name: 'F#', aliases: ['fs'] },
    { code: 'julia', name: 'Julia', aliases: ['jl'] },
    { code: 'matlab', name: 'MATLAB', aliases: ['m'] },
    { code: 'groovy', name: 'Groovy', aliases: [] },
    { code: 'vb', name: 'Visual Basic', aliases: ['vbnet'] },
    { code: 'objective-c', name: 'Objective-C', aliases: ['objc'] },
    { code: 'assembly', name: 'Assembly', aliases: ['asm'] },
    { code: 'toml', name: 'TOML', aliases: [] },
    { code: 'ini', name: 'INI', aliases: [] },
    { code: 'nginx', name: 'Nginx', aliases: [] },
    { code: 'apache', name: 'Apache', aliases: [] },
    { code: 'vim', name: 'Vim Script', aliases: ['vimscript'] },
    { code: 'diff', name: 'Diff', aliases: [] },
    { code: 'plaintext', name: 'Plain Text', aliases: ['text', 'txt'] }
  ];

  // Get the current selection range
  function getSelection() {
    if (!textareaRef.value) return null;
    
    const textarea = textareaRef.value;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    
    if (start === end) return null; // No selection
    
    return {
      start,
      end,
      text: textarea.value.substring(start, end)
    };
  }

  // Calculate position for language selector
  function calculateSelectorPosition() {
    if (!textareaRef.value) return;
    
    const textarea = textareaRef.value;
    const rect = textarea.getBoundingClientRect();
    
    // Create a temporary element to measure text dimensions
    const tempElement = document.createElement('div');
    tempElement.style.cssText = window.getComputedStyle(textarea).cssText;
    tempElement.style.position = 'absolute';
    tempElement.style.visibility = 'hidden';
    tempElement.style.whiteSpace = 'pre-wrap';
    tempElement.style.width = `${textarea.clientWidth}px`;
    tempElement.textContent = textarea.value.substring(0, selectedRange.value.start);
    document.body.appendChild(tempElement);
    
    // Get the position of the cursor
    const lines = tempElement.textContent.split('\n');
    const lastLineWidth = lines[lines.length - 1].length * 8; // Approximate char width
    const lineHeight = parseInt(window.getComputedStyle(textarea).lineHeight);
    const cursorY = (lines.length - 1) * lineHeight;
    
    document.body.removeChild(tempElement);
    
    // Position the selector above the selection
    languageSelectorPosition.value = {
      top: rect.top + cursorY - 40 + textarea.scrollTop,
      left: rect.left + Math.min(lastLineWidth, rect.width - 200)
    };
  }

  // Wrap selected text with code block
  function wrapWithCodeBlock(language = '') {
    if (!textareaRef.value) return;
    
    const textarea = textareaRef.value;
    const selection = getSelection();
    
    // Store original state for undo
    const originalValue = textarea.value;
    const originalSelectionStart = textarea.selectionStart;
    const originalSelectionEnd = textarea.selectionEnd;
    
    // Create undo point
    textarea.focus();
    document.execCommand('insertText', false, '');
    
    if (!selection) {
      // No selection, insert empty code block at cursor
      const cursorPos = textarea.selectionStart;
      const before = textarea.value.substring(0, cursorPos);
      const after = textarea.value.substring(cursorPos);
      
      // Ensure proper newlines
      const needNewlineBefore = cursorPos > 0 && before[before.length - 1] !== '\n';
      const prefix = needNewlineBefore ? '\n' : '';
      
      const codeBlock = `${prefix}\`\`\`${language}\n\n\`\`\``;
      
      // Use execCommand for better undo support
      textarea.setSelectionRange(cursorPos, cursorPos);
      document.execCommand('insertText', false, codeBlock);
      
      // Position cursor inside the code block
      const newCursorPos = cursorPos + prefix.length + 3 + language.length + 1;
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    } else {
      // Wrap selection
      const selectedText = selection.text;
      
      // Check if we need newlines
      const needNewlineBefore = selection.start > 0 && textarea.value[selection.start - 1] !== '\n';
      const needNewlineAfter = selection.end < textarea.value.length && textarea.value[selection.end] !== '\n';
      
      const prefix = needNewlineBefore ? '\n' : '';
      const suffix = needNewlineAfter ? '\n' : '';
      
      const wrappedText = `${prefix}\`\`\`${language}\n${selectedText}\n\`\`\`${suffix}`;
      
      // Use execCommand for better undo support
      textarea.setSelectionRange(selection.start, selection.end);
      document.execCommand('insertText', false, wrappedText);
      
      // Select the entire code block
      const newStart = selection.start;
      const newEnd = newStart + wrappedText.length;
      textarea.setSelectionRange(newStart, newEnd);
    }
    
    // Trigger input event to update v-model
    textarea.dispatchEvent(new Event('input', { bubbles: true }));
    
    // Focus back on textarea
    textarea.focus();
  }

  // Handle keyboard shortcut (Cmd/Ctrl+E)
  function handleKeyboardShortcut(event) {
    if ((event.metaKey || event.ctrlKey) && event.key === 'e') {
      event.preventDefault();
      
      const selection = getSelection();
      if (selection) {
        selectedRange.value = selection;
        calculateSelectorPosition();
        showLanguageSelector.value = true;
      } else {
        // No selection, just wrap with empty code block
        wrapWithCodeBlock('');
      }
    }
  }

  // Select language and wrap
  function selectLanguage(language) {
    wrapWithCodeBlock(language.code);
    showLanguageSelector.value = false;
    
    // Add to recent languages for quick access
    addToRecentLanguages(language);
  }

  // Cancel language selection
  function cancelSelection() {
    showLanguageSelector.value = false;
    textareaRef.value?.focus();
  }

  // Recent languages management
  const recentLanguages = ref(
    JSON.parse(localStorage.getItem('codeBlockRecentLanguages') || '[]')
  );

  function addToRecentLanguages(language) {
    const recent = recentLanguages.value.filter(l => l.code !== language.code);
    recent.unshift(language);
    recentLanguages.value = recent.slice(0, 5); // Keep only 5 recent
    localStorage.setItem('codeBlockRecentLanguages', JSON.stringify(recentLanguages.value));
  }

  // Get suggested languages (recent + popular)
  function getSuggestedLanguages() {
    const popularCodes = ['javascript', 'typescript', 'python', 'rust', 'go', 'java'];
    const popular = languages.filter(l => popularCodes.includes(l.code));
    
    // Combine recent and popular, removing duplicates
    const combined = [...recentLanguages.value];
    popular.forEach(lang => {
      if (!combined.find(l => l.code === lang.code)) {
        combined.push(lang);
      }
    });
    
    return combined.slice(0, 8);
  }

  // Search languages
  function searchLanguages(query) {
    if (!query) return getSuggestedLanguages();
    
    const lowerQuery = query.toLowerCase();
    return languages.filter(lang => 
      lang.name.toLowerCase().includes(lowerQuery) ||
      lang.code.toLowerCase().includes(lowerQuery) ||
      lang.aliases.some(alias => alias.toLowerCase().includes(lowerQuery))
    ).slice(0, 10);
  }

  // Check if text contains code patterns
  function detectCodePattern(text) {
    const codePatterns = [
      /function\s+\w+\s*\(/,
      /const\s+\w+\s*=/,
      /let\s+\w+\s*=/,
      /var\s+\w+\s*=/,
      /class\s+\w+/,
      /def\s+\w+\s*\(/,
      /if\s*\(/,
      /for\s*\(/,
      /while\s*\(/,
      /import\s+/,
      /export\s+/,
      /return\s+/,
      /{[\s\S]*}/,
      /\[[\s\S]*\]/,
      /=>/, 
      /\+\+/,
      /--/,
      /===/,
      /!=/
    ];
    
    return codePatterns.some(pattern => pattern.test(text));
  }

  // Auto-detect language from code content
  function detectLanguage(code) {
    const detectors = [
      { patterns: [/import\s+.*\s+from/, /export\s+(default\s+)?/, /const\s+\w+\s*=.*=>/, /\.jsx?$/], language: 'javascript' },
      { patterns: [/:\s*(string|number|boolean|any|void)/, /interface\s+\w+/, /type\s+\w+\s*=/, /\.tsx?$/], language: 'typescript' },
      { patterns: [/def\s+\w+.*:/, /import\s+\w+/, /if\s+__name__.*==.*__main__/, /\.py$/], language: 'python' },
      { patterns: [/fn\s+\w+.*->/, /let\s+mut\s+/, /use\s+\w+/, /impl\s+/, /\.rs$/], language: 'rust' },
      { patterns: [/func\s+\w+.*{/, /package\s+\w+/, /import\s*\(/, /\.go$/], language: 'go' },
      { patterns: [/public\s+class/, /private\s+\w+/, /System\.out\.print/, /\.java$/], language: 'java' },
      { patterns: [/#include\s*</, /int\s+main\s*\(/, /std::/, /\.cpp$/, /\.cc$/], language: 'cpp' },
      { patterns: [/<\?php/, /function\s+\w+\s*\(/, /\$\w+\s*=/, /\.php$/], language: 'php' },
      { patterns: [/def\s+\w+.*end/, /class\s+\w+.*end/, /puts\s+/, /\.rb$/], language: 'ruby' },
      { patterns: [/SELECT\s+.*FROM/i, /INSERT\s+INTO/i, /UPDATE\s+.*SET/i, /\.sql$/], language: 'sql' },
      { patterns: [/<html/, /<div/, /<span/, /<p/, /\.html$/], language: 'html' },
      { patterns: [/{\s*[\w-]+\s*:/, /\.\w+\s*{/, /#\w+\s*{/, /\.css$/], language: 'css' },
      { patterns: [/\$[\w-]+\s*:/, /@mixin\s+/, /@include\s+/, /\.scss$/, /\.sass$/], language: 'scss' },
      { patterns: [/^{[\s\S]*}$/, /"[\w-]+"\s*:/, /\.json$/], language: 'json' },
      { patterns: [/^[\w-]+:/, /^\s+-\s+/, /\.ya?ml$/], language: 'yaml' }
    ];
    
    for (const detector of detectors) {
      if (detector.patterns.some(pattern => pattern.test(code))) {
        return detector.language;
      }
    }
    
    return '';
  }

  return {
    showLanguageSelector,
    languageSelectorPosition,
    languages,
    recentLanguages,
    handleKeyboardShortcut,
    wrapWithCodeBlock,
    selectLanguage,
    cancelSelection,
    searchLanguages,
    getSuggestedLanguages,
    detectCodePattern,
    detectLanguage
  };
}