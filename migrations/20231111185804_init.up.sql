-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    "languages" (
        code VARCHAR(7) NOT NULL PRIMARY KEY,
        name VARCHAR(50) NOT NULL
    );

CREATE TABLE
    "users" (
        id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
        language_id VARCHAR(7) NOT NULL,
        email VARCHAR(255) NOT NULL UNIQUE,
        verified BOOLEAN NOT NULL DEFAULT FALSE,
        role VARCHAR(50) NOT NULL DEFAULT 'user',
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        CONSTRAINT fk_language
            FOREIGN KEY(language_id)
                REFERENCES languages(code)
    );

CREATE INDEX users_email_idx ON users (email);

CREATE TABLE
    "codes" (
        id UUID NOT NULL PRIMARY KEY,
        code VARCHAR(8) NOT NULL,
        tries SMALLINT NOT NULL DEFAULT 0,
        emitted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        CONSTRAINT fk_user
            FOREIGN KEY(id)
                REFERENCES users(id)
    );

CREATE TABLE
    "tokens" (
        user_id UUID NOT NULL,
        token_id UUID NOT NULL,
        is_valid BOOLEAN NOT NULL DEFAULT true,
        expiration TIMESTAMP WITH TIME ZONE NOT NULL,
        PRIMARY KEY (user_id, token_id)
    );


-- Insert data

INSERT INTO Languages (code, name)
VALUES ( 'fr', 'Français' );

INSERT INTO Languages (code, name)
VALUES ( 'en', 'English' );