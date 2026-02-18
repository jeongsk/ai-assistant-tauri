// Database Schema and Migrations

use rusqlite::Connection;
use rusqlite::Result;

const _SCHEMA_VERSION: i32 = 3;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Apply migrations
    if current_version < 1 {
        migrate_v1(conn)?;
    }

    if current_version < 2 {
        migrate_v2(conn)?;
    }

    if current_version < 3 {
        migrate_v3(conn)?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Conversations table
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
            content TEXT NOT NULL,
            metadata TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Folder permissions table
        CREATE TABLE IF NOT EXISTS folder_permissions (
            id TEXT PRIMARY KEY,
            path TEXT UNIQUE NOT NULL,
            level TEXT NOT NULL CHECK(level IN ('read', 'readwrite')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes for better query performance
        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at);
        CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (1);
        "#,
    )?;

    Ok(())
}

fn migrate_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Skills table
        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL CHECK(length(description) <= 500),
            prompt TEXT NOT NULL CHECK(length(prompt) <= 10240),
            tools TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Recipes table
        CREATE TABLE IF NOT EXISTS recipes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            version TEXT NOT NULL DEFAULT '1.0.0',
            steps TEXT NOT NULL,
            variables TEXT,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Recipe executions log
        CREATE TABLE IF NOT EXISTS recipe_executions (
            id TEXT PRIMARY KEY,
            recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'cancelled')),
            variables TEXT,
            result TEXT,
            error TEXT,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT
        );

        -- Sub-agent placeholder table (v0.3 preparation)
        CREATE TABLE IF NOT EXISTS sub_agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            role TEXT NOT NULL,
            system_prompt TEXT,
            tools TEXT NOT NULL DEFAULT '[]',
            config TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);
        CREATE INDEX IF NOT EXISTS idx_recipes_name ON recipes(name);
        CREATE INDEX IF NOT EXISTS idx_recipe_executions_recipe ON recipe_executions(recipe_id);
        CREATE INDEX IF NOT EXISTS idx_recipe_executions_status ON recipe_executions(status);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (2);
        "#,
    )?;

    Ok(())
}

fn migrate_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Extend sub_agents table with task tracking
        ALTER TABLE sub_agents ADD COLUMN status TEXT NOT NULL DEFAULT 'idle';
        ALTER TABLE sub_agents ADD COLUMN task TEXT;
        ALTER TABLE sub_agents ADD COLUMN result TEXT;
        ALTER TABLE sub_agents ADD COLUMN error TEXT;
        ALTER TABLE sub_agents ADD COLUMN completed_at TEXT;

        -- Cron jobs table
        CREATE TABLE IF NOT EXISTS cron_jobs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            schedule TEXT NOT NULL,
            job_type TEXT NOT NULL CHECK(job_type IN ('skill', 'recipe', 'prompt', 'system')),
            config TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            last_run TEXT,
            next_run TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Job executions log
        CREATE TABLE IF NOT EXISTS job_executions (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL REFERENCES cron_jobs(id) ON DELETE CASCADE,
            status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'cancelled')),
            result TEXT,
            error TEXT,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            completed_at TEXT
        );

        -- Routing rules table
        CREATE TABLE IF NOT EXISTS routing_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            condition TEXT NOT NULL DEFAULT '{}',
            provider TEXT NOT NULL,
            model TEXT,
            priority INTEGER NOT NULL DEFAULT 0,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_sub_agents_status ON sub_agents(status);
        CREATE INDEX IF NOT EXISTS idx_cron_jobs_enabled ON cron_jobs(enabled);
        CREATE INDEX IF NOT EXISTS idx_cron_jobs_next_run ON cron_jobs(next_run);
        CREATE INDEX IF NOT EXISTS idx_job_executions_job ON job_executions(job_id);
        CREATE INDEX IF NOT EXISTS idx_routing_rules_priority ON routing_rules(priority);

        -- Insert default routing rules
        INSERT INTO routing_rules (id, name, description, condition, provider, model, priority) VALUES
            ('rule-code', 'Code Generation', 'Route coding tasks to capable models', '{"taskTypes":["coding"]}', 'anthropic', 'claude-3-sonnet', 100),
            ('rule-chat', 'Simple Chat', 'Route simple chat to fast models', '{"taskTypes":["chat"],"maxTokens":500}', 'openai', 'gpt-3.5-turbo', 50),
            ('rule-analysis', 'Analysis', 'Route analysis to reasoning models', '{"taskTypes":["analysis"]}', 'openai', 'gpt-4', 80),
            ('rule-creative', 'Creative Writing', 'Route creative tasks to Claude', '{"taskTypes":["creative"]}', 'anthropic', 'claude-3-opus', 90);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (3);
        "#,
    )?;

    Ok(())
}
