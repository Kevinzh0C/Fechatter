<template>
  <div class="code-highlight-demo">
    <h2 class="demo-title">Enhanced Code Highlighting</h2>
    <p class="demo-description">Industry-grade syntax highlighting inspired by VS Code, GitHub, and Discord</p>

    <div class="demo-section">
      <h3>✨ Features</h3>
      <ul class="feature-list">
        <li>🌈 20+ Programming Languages</li>
        <li>Auto Language Detection</li>
        <li>📋 One-Click Copy</li>
        <li>🌙 Dark Mode Support</li>
        <li>📱 Responsive Design</li>
        <li>Inline Code Support</li>
      </ul>
    </div>

    <div class="demo-section">
      <h3>💻 Code Block Examples</h3>

      <!-- Python Example -->
      <div class="example">
        <h4>Python Migration Function</h4>
        <EnhancedMessageContent :content="pythonExample" :enable-markdown="true" />
      </div>

      <!-- JavaScript Example -->
      <div class="example">
        <h4>JavaScript Implementation</h4>
        <EnhancedMessageContent :content="javascriptExample" :enable-markdown="true" />
      </div>

      <!-- Inline Code Example -->
      <div class="example">
        <h4>Inline Code Support</h4>
        <EnhancedMessageContent :content="inlineExample" :enable-markdown="true" />
      </div>

      <!-- Multi-language Example -->
      <div class="example">
        <h4>Multiple Languages in One Message</h4>
        <EnhancedMessageContent :content="multiLanguageExample" :enable-markdown="true" />
      </div>
    </div>
  </div>
</template>

<script setup>
import EnhancedMessageContent from './EnhancedMessageContent.vue';

const pythonExample = `Here's the Python migration function:

\`\`\`python
def migrate_users(batch_size=1000):
    offset = 0
    while True:
        users = fetch_old_users(offset, batch_size)
        if not users:
            break
        insert_new_users(users)
        offset += batch_size
\`\`\`

This handles large datasets efficiently with **batching**.`;

const javascriptExample = `Modern JavaScript implementation:

\`\`\`javascript
async function migrateUsers(batchSize = 1000) {
    let offset = 0;
    while (true) {
        const users = await fetchOldUsers(offset, batchSize);
        if (!users.length) break;
        
        await insertNewUsers(users);
        offset += batchSize;
    }
}
\`\`\`

Uses *async/await* for better error handling.`;

const inlineExample = `You can use inline code like \`batch_size=1000\` for parameters, \`async/await\` for promises, and \`fetch()\` for API calls in your documentation.`;

const multiLanguageExample = `Database migration with multiple technologies:

**SQL Schema:**
\`\`\`sql
CREATE TABLE users_new AS 
SELECT id, name, email, created_at 
FROM users_old 
WHERE active = true;
\`\`\`

**Rust Implementation:**
\`\`\`rust
fn migrate_batch(offset: usize, limit: usize) -> Result<usize, Error> {
    let users = fetch_users(offset, limit)?;
    insert_users_batch(&users)?;
    Ok(users.len())
}
\`\`\`

**Configuration (YAML):**
\`\`\`yaml
migration:
  batch_size: 1000
  parallel_workers: 4
  retry_attempts: 3
\`\`\``;
</script>

<style scoped>
.code-highlight-demo {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

.demo-title {
  font-size: 2rem;
  font-weight: 700;
  color: #24292f;
  margin-bottom: 8px;
  text-align: center;
}

.demo-description {
  font-size: 1.1rem;
  color: #656d76;
  text-align: center;
  margin-bottom: 32px;
  line-height: 1.5;
}

.demo-section {
  margin-bottom: 32px;
}

.demo-section h3 {
  font-size: 1.5rem;
  font-weight: 600;
  color: #24292f;
  margin-bottom: 16px;
  border-bottom: 2px solid #e1e4e8;
  padding-bottom: 8px;
}

.feature-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 8px;
  list-style: none;
  padding: 0;
  margin-bottom: 24px;
}

.feature-list li {
  padding: 8px 12px;
  background: #f6f8fa;
  border: 1px solid #e1e4e8;
  border-radius: 6px;
  font-weight: 500;
  color: #24292f;
  transition: all 0.2s ease;
}

.feature-list li:hover {
  background: #f3f4f6;
  border-color: #d0d7de;
  transform: translateY(-1px);
}

.example {
  margin-bottom: 24px;
  padding: 20px;
  background: #fafbfc;
  border: 1px solid #e1e4e8;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.example h4 {
  font-size: 1.2rem;
  font-weight: 600;
  color: #24292f;
  margin: 0 0 16px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.example h4::before {
  content: "";
  font-size: 1rem;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .code-highlight-demo {
    color: #e6edf3;
  }

  .demo-title {
    color: #e6edf3;
  }

  .demo-description {
    color: #8b949e;
  }

  .demo-section h3 {
    color: #e6edf3;
    border-color: #30363d;
  }

  .feature-list li {
    background: #21262d;
    border-color: #30363d;
    color: #e6edf3;
  }

  .feature-list li:hover {
    background: #30363d;
    border-color: #444c56;
  }

  .example {
    background: #0d1117;
    border-color: #30363d;
  }

  .example h4 {
    color: #e6edf3;
  }
}

/* Responsive design */
@media (max-width: 768px) {
  .code-highlight-demo {
    padding: 16px;
  }

  .demo-title {
    font-size: 1.5rem;
  }

  .demo-description {
    font-size: 1rem;
  }

  .feature-list {
    grid-template-columns: 1fr;
  }

  .example {
    padding: 16px;
  }
}
</style>