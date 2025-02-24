// Test utility for code highlighting functionality
export const testCodeSamples = {
  python: `def migrate_users(batch_size=1000):
    offset = 0
    while True:
        users = fetch_old_users(offset, batch_size)
        if not users:
            break
        insert_new_users(users)
        offset += batch_size`,

  javascript: `function migrateUsers(batchSize = 1000) {
    let offset = 0;
    while (true) {
        const users = fetchOldUsers(offset, batchSize);
        if (!users.length) {
            break;
        }
        insertNewUsers(users);
        offset += batchSize;
    }
}`,

  sql: `SELECT u.id, u.name, u.email, p.profile_data
FROM users u
LEFT JOIN profiles p ON u.id = p.user_id
WHERE u.created_at >= '2024-01-01'
ORDER BY u.created_at DESC
LIMIT 100;`,

  json: `{
  "name": "fechatter",
  "version": "1.0.0",
  "dependencies": {
    "highlight.js": "^11.11.1",
    "vue": "^3.3.0"
  }
}`,

  rust: `fn migrate_users(batch_size: usize) -> Result<(), Box<dyn Error>> {
    let mut offset = 0;
    loop {
        let users = fetch_old_users(offset, batch_size)?;
        if users.is_empty() {
            break;
        }
        insert_new_users(&users)?;
        offset += batch_size;
    }
    Ok(())
}`
};

// Test messages with different code formats
export const testMessages = [
  {
    id: 1,
    content: `Here's the Python migration function:

\`\`\`python
${testCodeSamples.python}
\`\`\`

This should handle large datasets efficiently.`,
    sender: { id: 1, fullname: 'Developer' },
    created_at: new Date().toISOString()
  },
  {
    id: 2,
    content: `You can also use inline code like \`batch_size=1000\` for parameters.`,
    sender: { id: 2, fullname: 'Code Reviewer' },
    created_at: new Date().toISOString()
  },
  {
    id: 3,
    content: `Here's the same logic in JavaScript:

\`\`\`js
${testCodeSamples.javascript}
\`\`\`

And in Rust:

\`\`\`rust
${testCodeSamples.rust}
\`\`\``,
    sender: { id: 3, fullname: 'Polyglot Dev' },
    created_at: new Date().toISOString()
  }
];

export default { testCodeSamples, testMessages }; 