CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS user_roles (
    id SMALLSERIAL PRIMARY KEY,
    code VARCHAR(40) NOT NULL UNIQUE,
    name VARCHAR(80) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS app_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id SMALLINT NOT NULL REFERENCES user_roles(id),
    username VARCHAR(64) NOT NULL UNIQUE,
    password_plain VARCHAR(128) NOT NULL,
    full_name VARCHAR(140) NOT NULL,
    email VARCHAR(120) UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    actor_user_id UUID REFERENCES app_users(id) ON DELETE SET NULL,
    action_type VARCHAR(40) NOT NULL,
    entity_name VARCHAR(60) NOT NULL,
    entity_id UUID,
    details TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS property_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS property_statuses (
    id SERIAL PRIMARY KEY,
    code VARCHAR(40) NOT NULL UNIQUE,
    name VARCHAR(80) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS property_conditions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(80) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS districts (
    id SERIAL PRIMARY KEY,
    city VARCHAR(80) NOT NULL DEFAULT 'Минск',
    name VARCHAR(80) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS owners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name VARCHAR(140) NOT NULL,
    phone VARCHAR(40) NOT NULL UNIQUE,
    email VARCHAR(120) UNIQUE,
    passport_no VARCHAR(40) UNIQUE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS client_types (
    id SERIAL PRIMARY KEY,
    code VARCHAR(40) NOT NULL UNIQUE,
    name VARCHAR(80) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_type_id INTEGER NOT NULL REFERENCES client_types(id),
    full_name VARCHAR(140) NOT NULL,
    phone VARCHAR(40) NOT NULL,
    email VARCHAR(120),
    budget NUMERIC(14, 2) CHECK (budget IS NULL OR budget >= 0),
    preferred_category_id INTEGER REFERENCES property_categories(id),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_clients_name_phone UNIQUE (full_name, phone)
);

CREATE TABLE IF NOT EXISTS properties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES owners(id),
    category_id INTEGER NOT NULL REFERENCES property_categories(id),
    status_id INTEGER NOT NULL REFERENCES property_statuses(id),
    condition_id INTEGER NOT NULL REFERENCES property_conditions(id),
    district_id INTEGER NOT NULL REFERENCES districts(id),
    title VARCHAR(160) NOT NULL,
    area NUMERIC(10, 2) NOT NULL CHECK (area > 0),
    price NUMERIC(14, 2) NOT NULL CHECK (price >= 0),
    address TEXT NOT NULL,
    description TEXT,
    listed_at DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_properties_owner_address UNIQUE (owner_id, address)
);

CREATE TABLE IF NOT EXISTS deal_types (
    id SERIAL PRIMARY KEY,
    code VARCHAR(40) NOT NULL UNIQUE,
    name VARCHAR(80) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS deal_statuses (
    id SERIAL PRIMARY KEY,
    code VARCHAR(40) NOT NULL UNIQUE,
    name VARCHAR(80) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL REFERENCES properties(id),
    client_id UUID NOT NULL REFERENCES clients(id),
    manager_id UUID NOT NULL REFERENCES app_users(id),
    deal_type_id INTEGER NOT NULL REFERENCES deal_types(id),
    status_id INTEGER NOT NULL REFERENCES deal_statuses(id),
    amount NUMERIC(14, 2) NOT NULL CHECK (amount >= 0),
    commission NUMERIC(14, 2) NOT NULL DEFAULT 0 CHECK (commission >= 0),
    start_date DATE NOT NULL DEFAULT CURRENT_DATE,
    end_date DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deal_id UUID NOT NULL REFERENCES deals(id) ON DELETE CASCADE,
    paid_at DATE NOT NULL DEFAULT CURRENT_DATE,
    amount NUMERIC(14, 2) NOT NULL CHECK (amount > 0),
    payment_method VARCHAR(40) NOT NULL,
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor_user_id ON audit_log (actor_user_id);
CREATE INDEX IF NOT EXISTS idx_clients_client_type_id ON clients (client_type_id);
CREATE INDEX IF NOT EXISTS idx_clients_preferred_category_id ON clients (preferred_category_id);
CREATE INDEX IF NOT EXISTS idx_deals_manager_id ON deals (manager_id);
CREATE INDEX IF NOT EXISTS idx_deals_property_id ON deals (property_id);
CREATE INDEX IF NOT EXISTS idx_deals_status_id ON deals (status_id);
CREATE INDEX IF NOT EXISTS idx_deals_type_id ON deals (deal_type_id);
CREATE INDEX IF NOT EXISTS idx_payments_deal_id ON payments (deal_id);
CREATE INDEX IF NOT EXISTS idx_payments_paid_at ON payments (paid_at);
CREATE INDEX IF NOT EXISTS idx_properties_category_id ON properties (category_id);
CREATE INDEX IF NOT EXISTS idx_properties_district_id ON properties (district_id);
CREATE INDEX IF NOT EXISTS idx_properties_owner_id ON properties (owner_id);
CREATE INDEX IF NOT EXISTS idx_properties_status_id ON properties (status_id);
