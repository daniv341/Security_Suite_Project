CREATE TYPE action_log AS ENUM (
    'Create',
    'Update',
    'Delete',
    'Get',
    'GetAll',
    'Login',
    'Logout'
);

CREATE TYPE resource_log AS ENUM (
    'Folder',
    'File',
    'User'
);

CREATE TABLE IF NOT EXISTS logs (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action action_log NOT NULL,
    resource resource_log NOT NULL,
    resource_id UUID NOT NULL,
    details VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_logs_user_id ON logs (user_id);
CREATE INDEX IF NOT EXISTS idx_logs_action ON logs (action);
CREATE INDEX IF NOT EXISTS idx_logs_resource ON logs (resource);