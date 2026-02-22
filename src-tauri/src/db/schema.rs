// Database Schema and Migrations

use rusqlite::Connection;
use rusqlite::Result;

const _SCHEMA_VERSION: i32 = 7;

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

    if current_version < 4 {
        migrate_v4(conn)?;
    }

    if current_version < 5 {
        migrate_v5(conn)?;
    }

    if current_version < 6 {
        migrate_v6(conn)?;
    }

    if current_version < 7 {
        migrate_v7(conn)?;
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

fn migrate_v4(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Memory tables
        CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL CHECK(type IN ('episodic', 'semantic', 'procedural')),
            content TEXT NOT NULL,
            embedding BLOB,
            metadata TEXT,
            importance REAL DEFAULT 0.5,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_accessed TEXT
        );

        CREATE TABLE IF NOT EXISTS user_patterns (
            id TEXT PRIMARY KEY,
            pattern_type TEXT NOT NULL,
            pattern_data TEXT NOT NULL,
            confidence REAL DEFAULT 0.0,
            sample_count INTEGER DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Plugin table
        CREATE TABLE IF NOT EXISTS plugins (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            version TEXT NOT NULL,
            manifest TEXT NOT NULL,
            permissions TEXT NOT NULL DEFAULT '[]',
            enabled INTEGER NOT NULL DEFAULT 0,
            installed_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Collaboration tables
        CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT 'general',
            content TEXT NOT NULL,
            visibility TEXT NOT NULL DEFAULT 'private' CHECK(visibility IN ('private', 'public', 'team')),
            version TEXT NOT NULL DEFAULT '1.0.0',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS shared_workflows (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            steps TEXT NOT NULL,
            owner_id TEXT,
            visibility TEXT NOT NULL DEFAULT 'private',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Voice settings
        CREATE TABLE IF NOT EXISTS voice_settings (
            id TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL DEFAULT 0,
            stt_model TEXT NOT NULL DEFAULT 'base',
            tts_voice TEXT NOT NULL DEFAULT 'default',
            language TEXT NOT NULL DEFAULT 'en',
            wake_word TEXT,
            vad_sensitivity REAL DEFAULT 0.5,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(type);
        CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance);
        CREATE INDEX IF NOT EXISTS idx_user_patterns_type ON user_patterns(pattern_type);
        CREATE INDEX IF NOT EXISTS idx_plugins_name ON plugins(name);
        CREATE INDEX IF NOT EXISTS idx_templates_category ON templates(category);
        CREATE INDEX IF NOT EXISTS idx_templates_visibility ON templates(visibility);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (4);
        "#,
    )?;

    Ok(())
}

/// Migration v5: Add tables for encrypted credentials
///
/// This migration:
/// 1. Adds `encrypted_password` column placeholder to database_connections
/// 2. Creates `cloud_storages` table for cloud provider credentials
/// 3. Creates `git_repositories` table for git SSH key storage
fn migrate_v5(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Add encrypted_password column to database_connections (may fail if already exists)
        ALTER TABLE database_connections ADD COLUMN encrypted_password TEXT;

        -- Create cloud_storages table
        CREATE TABLE IF NOT EXISTS cloud_storages (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            provider TEXT NOT NULL,
            bucket TEXT NOT NULL,
            region TEXT,
            encrypted_credentials TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Create git_repositories table
        CREATE TABLE IF NOT EXISTS git_repositories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            path TEXT NOT NULL,
            encrypted_ssh_key TEXT,
            user_name TEXT,
            user_email TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_cloud_storages_name ON cloud_storages(name);
        CREATE INDEX IF NOT EXISTS idx_git_repositories_name ON git_repositories(name);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (5);
        "#,
    )?;

    tracing::info!("Database migration v5 completed");

    Ok(())
}

/// Migration v6: Add tables for template versioning and sharing
///
/// This migration:
/// 1. Creates `template_versions` table for template version history
/// 2. Creates `template_shares` table for team template sharing
fn migrate_v6(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Create template_versions table for version history
        CREATE TABLE IF NOT EXISTS template_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            content TEXT NOT NULL,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(template_id, version)
        );

        -- Create template_shares table for team sharing
        CREATE TABLE IF NOT EXISTS template_shares (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            team_id TEXT NOT NULL,
            permissions TEXT NOT NULL,
            shared_by TEXT NOT NULL,
            shared_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(template_id, team_id)
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_template_versions_template_id ON template_versions(template_id);
        CREATE INDEX IF NOT EXISTS idx_template_versions_version ON template_versions(version);
        CREATE INDEX IF NOT EXISTS idx_template_shares_template_id ON template_shares(template_id);
        CREATE INDEX IF NOT EXISTS idx_template_shares_team_id ON template_shares(team_id);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (6);
        "#,
    )?;

    tracing::info!("Database migration v6 completed");

    Ok(())
}

/// Migration v7: Add tables for voice patterns and conversations
///
/// This migration:
/// 1. Creates `voice_patterns` table for user-defined voice command patterns
/// 2. Creates `voice_conversations` table for multi-turn voice conversation history
fn migrate_v7(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Create voice_patterns table for custom voice command patterns
        CREATE TABLE IF NOT EXISTS voice_patterns (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            pattern TEXT NOT NULL,
            action_type TEXT NOT NULL CHECK(action_type IN ('execute_skill', 'run_recipe', 'send_message', 'open_feature', 'search')),
            target_id TEXT,
            language TEXT NOT NULL DEFAULT 'en',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Create voice_conversations table for multi-turn conversation history
        CREATE TABLE IF NOT EXISTS voice_conversations (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
            content TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT 'en',
            audio_transcribed INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_voice_patterns_language ON voice_patterns(language);
        CREATE INDEX IF NOT EXISTS idx_voice_patterns_action_type ON voice_patterns(action_type);
        CREATE INDEX IF NOT EXISTS idx_voice_conversations_session_id ON voice_conversations(session_id);
        CREATE INDEX IF NOT EXISTS idx_voice_conversations_created_at ON voice_conversations(created_at);

        -- Record migration
        INSERT INTO schema_migrations (version) VALUES (7);
        "#,
    )?;

    tracing::info!("Database migration v7 completed");

    Ok(())
}
