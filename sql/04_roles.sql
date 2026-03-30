DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'realty_admin') THEN
        CREATE ROLE realty_admin LOGIN PASSWORD 'Admin123!';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'realty_realtor') THEN
        CREATE ROLE realty_realtor LOGIN PASSWORD 'Realtor123!';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'realty_guest') THEN
        CREATE ROLE realty_guest LOGIN PASSWORD 'Guest123!';
    END IF;
END
$$;

DO $$
BEGIN
    EXECUTE format('GRANT CONNECT ON DATABASE %I TO realty_admin, realty_realtor, realty_guest', current_database());
END
$$;

GRANT USAGE ON SCHEMA public TO realty_admin, realty_realtor, realty_guest;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO realty_guest;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO realty_realtor;
GRANT INSERT, UPDATE, DELETE ON owners, clients, properties, deals, payments, audit_log TO realty_realtor;
GRANT INSERT, UPDATE, DELETE, SELECT ON ALL TABLES IN SCHEMA public TO realty_admin;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO realty_admin, realty_realtor, realty_guest;
GRANT EXECUTE ON FUNCTION calculate_paid_amount(UUID) TO realty_admin, realty_realtor, realty_guest;
GRANT EXECUTE ON FUNCTION calculate_deal_commission(INTEGER, NUMERIC) TO realty_admin, realty_realtor, realty_guest;
GRANT EXECUTE ON FUNCTION search_properties(TEXT, INTEGER, INTEGER, INTEGER) TO realty_admin, realty_realtor, realty_guest;
GRANT EXECUTE ON FUNCTION manager_sales_report(DATE, DATE) TO realty_admin, realty_realtor, realty_guest;

ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO realty_guest;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO realty_realtor;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO realty_admin;
