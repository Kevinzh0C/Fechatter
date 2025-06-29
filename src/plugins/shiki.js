// Get or create a highlighter instance
export async function getHighlighter(theme = 'dark') {
  return createShikiHighlighter({ theme });
}

// Resolve language from alias or file extension 