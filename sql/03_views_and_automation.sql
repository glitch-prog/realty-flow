CREATE OR REPLACE FUNCTION touch_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_touch_user_roles ON user_roles;
CREATE TRIGGER trg_touch_user_roles BEFORE UPDATE ON user_roles FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_app_users ON app_users;
CREATE TRIGGER trg_touch_app_users BEFORE UPDATE ON app_users FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_property_categories ON property_categories;
CREATE TRIGGER trg_touch_property_categories BEFORE UPDATE ON property_categories FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_property_statuses ON property_statuses;
CREATE TRIGGER trg_touch_property_statuses BEFORE UPDATE ON property_statuses FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_property_conditions ON property_conditions;
CREATE TRIGGER trg_touch_property_conditions BEFORE UPDATE ON property_conditions FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_districts ON districts;
CREATE TRIGGER trg_touch_districts BEFORE UPDATE ON districts FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_owners ON owners;
CREATE TRIGGER trg_touch_owners BEFORE UPDATE ON owners FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_clients ON clients;
CREATE TRIGGER trg_touch_clients BEFORE UPDATE ON clients FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_properties ON properties;
CREATE TRIGGER trg_touch_properties BEFORE UPDATE ON properties FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

DROP TRIGGER IF EXISTS trg_touch_deals ON deals;
CREATE TRIGGER trg_touch_deals BEFORE UPDATE ON deals FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

CREATE OR REPLACE FUNCTION write_audit_log()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    entity_uuid UUID;
BEGIN
    entity_uuid := COALESCE(NEW.id, OLD.id);

    INSERT INTO audit_log (actor_user_id, action_type, entity_name, entity_id, details)
    VALUES (
        NULL,
        TG_OP,
        TG_TABLE_NAME,
        entity_uuid,
        jsonb_build_object(
            'old', COALESCE(to_jsonb(OLD), '{}'::jsonb),
            'new', COALESCE(to_jsonb(NEW), '{}'::jsonb)
        )::text
    );

    RETURN COALESCE(NEW, OLD);
END;
$$;

DROP TRIGGER IF EXISTS trg_audit_app_users ON app_users;
CREATE TRIGGER trg_audit_app_users AFTER INSERT OR UPDATE OR DELETE ON app_users FOR EACH ROW EXECUTE FUNCTION write_audit_log();

DROP TRIGGER IF EXISTS trg_audit_owners ON owners;
CREATE TRIGGER trg_audit_owners AFTER INSERT OR UPDATE OR DELETE ON owners FOR EACH ROW EXECUTE FUNCTION write_audit_log();

DROP TRIGGER IF EXISTS trg_audit_clients ON clients;
CREATE TRIGGER trg_audit_clients AFTER INSERT OR UPDATE OR DELETE ON clients FOR EACH ROW EXECUTE FUNCTION write_audit_log();

DROP TRIGGER IF EXISTS trg_audit_properties ON properties;
CREATE TRIGGER trg_audit_properties AFTER INSERT OR UPDATE OR DELETE ON properties FOR EACH ROW EXECUTE FUNCTION write_audit_log();

DROP TRIGGER IF EXISTS trg_audit_deals ON deals;
CREATE TRIGGER trg_audit_deals AFTER INSERT OR UPDATE OR DELETE ON deals FOR EACH ROW EXECUTE FUNCTION write_audit_log();

DROP TRIGGER IF EXISTS trg_audit_payments ON payments;
CREATE TRIGGER trg_audit_payments AFTER INSERT OR UPDATE OR DELETE ON payments FOR EACH ROW EXECUTE FUNCTION write_audit_log();

CREATE OR REPLACE FUNCTION calculate_deal_commission(p_deal_type_id INTEGER, p_amount NUMERIC)
RETURNS NUMERIC
LANGUAGE plpgsql
AS $$
DECLARE
    deal_type_code TEXT;
    rate NUMERIC := 0;
BEGIN
    SELECT code INTO deal_type_code
    FROM deal_types
    WHERE id = p_deal_type_id;

    IF deal_type_code = 'sale' THEN
        rate := 0.03;
    ELSIF deal_type_code = 'rent' THEN
        rate := 0.10;
    END IF;

    RETURN ROUND(COALESCE(p_amount, 0) * rate, 2);
END;
$$;

CREATE OR REPLACE FUNCTION is_property_available(
    p_property_id UUID,
    p_deal_type_id INTEGER,
    p_start_date DATE,
    p_end_date DATE,
    p_current_deal_id UUID DEFAULT NULL
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
DECLARE
    deal_type_code TEXT;
BEGIN
    SELECT code INTO deal_type_code
    FROM deal_types
    WHERE id = p_deal_type_id;

    IF deal_type_code = 'sale' THEN
        RETURN NOT EXISTS (
            SELECT 1
            FROM deals d
            JOIN deal_statuses ds ON ds.id = d.status_id
            JOIN deal_types dt ON dt.id = d.deal_type_id
            WHERE d.property_id = p_property_id
              AND dt.code = 'sale'
              AND ds.code IN ('registered', 'in_progress', 'completed')
              AND (p_current_deal_id IS NULL OR d.id <> p_current_deal_id)
        );
    END IF;

    IF deal_type_code = 'rent' THEN
        RETURN NOT EXISTS (
            SELECT 1
            FROM deals d
            JOIN deal_statuses ds ON ds.id = d.status_id
            JOIN deal_types dt ON dt.id = d.deal_type_id
            WHERE d.property_id = p_property_id
              AND dt.code = 'rent'
              AND ds.code IN ('registered', 'in_progress', 'completed')
              AND daterange(d.start_date, COALESCE(d.end_date, 'infinity'::date), '[]')
                  && daterange(p_start_date, COALESCE(p_end_date, 'infinity'::date), '[]')
              AND (p_current_deal_id IS NULL OR d.id <> p_current_deal_id)
        )
        AND NOT EXISTS (
            SELECT 1
            FROM deals d
            JOIN deal_statuses ds ON ds.id = d.status_id
            JOIN deal_types dt ON dt.id = d.deal_type_id
            WHERE d.property_id = p_property_id
              AND dt.code = 'sale'
              AND ds.code = 'completed'
              AND (p_current_deal_id IS NULL OR d.id <> p_current_deal_id)
        );
    END IF;

    RETURN TRUE;
END;
$$;

CREATE OR REPLACE FUNCTION validate_deal_before_save()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.end_date IS NOT NULL AND NEW.end_date < NEW.start_date THEN
        RAISE EXCEPTION 'Дата окончания сделки не может быть раньше даты начала';
    END IF;

    IF NOT is_property_available(NEW.property_id, NEW.deal_type_id, NEW.start_date, NEW.end_date, COALESCE(NEW.id, NULL)) THEN
        RAISE EXCEPTION 'Выбранный объект недоступен на указанный период или уже имеет конфликтующую сделку';
    END IF;

    NEW.commission := calculate_deal_commission(NEW.deal_type_id, NEW.amount);
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_validate_deal_before_save ON deals;
CREATE TRIGGER trg_validate_deal_before_save
BEFORE INSERT OR UPDATE ON deals
FOR EACH ROW
EXECUTE FUNCTION validate_deal_before_save();

CREATE OR REPLACE FUNCTION sync_property_status_from_deal()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    deal_type_code TEXT;
    deal_status_code TEXT;
    target_status_code TEXT;
BEGIN
    SELECT dt.code, ds.code
      INTO deal_type_code, deal_status_code
    FROM deal_types dt, deal_statuses ds
    WHERE dt.id = NEW.deal_type_id
      AND ds.id = NEW.status_id;

    IF deal_status_code IN ('registered', 'in_progress') THEN
        target_status_code := 'in_deal';
    ELSIF deal_status_code = 'completed' AND deal_type_code = 'sale' THEN
        target_status_code := 'sold';
    ELSIF deal_status_code = 'completed' AND deal_type_code = 'rent' THEN
        target_status_code := 'rented';
    ELSIF deal_status_code = 'cancelled' THEN
        target_status_code := 'free';
    END IF;

    IF target_status_code IS NOT NULL THEN
        UPDATE properties
        SET status_id = (
            SELECT id
            FROM property_statuses
            WHERE code = target_status_code
        )
        WHERE id = NEW.property_id;
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_sync_property_status_from_deal ON deals;
CREATE TRIGGER trg_sync_property_status_from_deal
AFTER INSERT OR UPDATE ON deals
FOR EACH ROW
EXECUTE FUNCTION sync_property_status_from_deal();

CREATE OR REPLACE VIEW vw_property_catalog AS
SELECT
    p.id AS property_id,
    p.title AS property_title,
    pc.name AS category_name,
    ps.code AS status_code,
    ps.name AS status_name,
    cond.name AS condition_name,
    d.city,
    d.name AS district_name,
    o.full_name AS owner_name,
    p.area,
    p.price,
    p.address,
    p.listed_at
FROM properties p
JOIN property_categories pc ON pc.id = p.category_id
JOIN property_statuses ps ON ps.id = p.status_id
JOIN property_conditions cond ON cond.id = p.condition_id
JOIN districts d ON d.id = p.district_id
JOIN owners o ON o.id = p.owner_id;

CREATE OR REPLACE VIEW vw_client_preferences AS
SELECT
    c.id AS client_id,
    c.full_name,
    ct.name AS client_type_name,
    c.phone,
    c.email,
    c.budget,
    COALESCE(pc.name, 'Без предпочтения') AS preferred_category_name,
    c.created_at
FROM clients c
JOIN client_types ct ON ct.id = c.client_type_id
LEFT JOIN property_categories pc ON pc.id = c.preferred_category_id;

CREATE OR REPLACE VIEW vw_active_deals AS
SELECT
    deal.id AS deal_id,
    prop.title AS property_title,
    cli.full_name AS client_name,
    usr.full_name AS manager_name,
    dt.name AS deal_type_name,
    ds.name AS deal_status_name,
    deal.amount,
    deal.commission,
    deal.start_date,
    deal.end_date
FROM deals deal
JOIN properties prop ON prop.id = deal.property_id
JOIN clients cli ON cli.id = deal.client_id
JOIN app_users usr ON usr.id = deal.manager_id
JOIN deal_types dt ON dt.id = deal.deal_type_id
JOIN deal_statuses ds ON ds.id = deal.status_id
WHERE ds.code IN ('registered', 'in_progress', 'completed');

CREATE OR REPLACE VIEW vw_owner_portfolio AS
SELECT
    o.id AS owner_id,
    o.full_name AS owner_name,
    COUNT(p.id) AS properties_count,
    COUNT(d.id) FILTER (WHERE ds.code IN ('registered', 'in_progress')) AS active_deals,
    COALESCE(SUM(p.price), 0) AS total_portfolio_value
FROM owners o
LEFT JOIN properties p ON p.owner_id = o.id
LEFT JOIN deals d ON d.property_id = p.id
LEFT JOIN deal_statuses ds ON ds.id = d.status_id
GROUP BY o.id, o.full_name;

CREATE OR REPLACE VIEW vw_sales_summary_by_month AS
SELECT
    to_char(date_trunc('month', d.start_date), 'YYYY-MM') AS period_label,
    COUNT(d.id) FILTER (WHERE ds.code = 'completed') AS completed_deals,
    COALESCE(SUM(d.amount) FILTER (WHERE ds.code = 'completed'), 0) AS total_amount,
    COALESCE(SUM(d.commission) FILTER (WHERE ds.code = 'completed'), 0) AS total_commission,
    COALESCE(SUM(p.amount), 0) AS payments_received
FROM deals d
JOIN deal_statuses ds ON ds.id = d.status_id
LEFT JOIN payments p ON p.deal_id = d.id
GROUP BY date_trunc('month', d.start_date)
ORDER BY date_trunc('month', d.start_date);

CREATE OR REPLACE VIEW vw_manager_performance AS
SELECT
    u.id AS manager_id,
    u.full_name AS manager_name,
    COUNT(d.id) FILTER (WHERE ds.code = 'completed') AS completed_deals,
    COALESCE(SUM(d.amount) FILTER (WHERE ds.code = 'completed'), 0) AS total_amount,
    COALESCE(SUM(d.commission) FILTER (WHERE ds.code = 'completed'), 0) AS total_commission
FROM app_users u
LEFT JOIN deals d ON d.manager_id = u.id
LEFT JOIN deal_statuses ds ON ds.id = d.status_id
GROUP BY u.id, u.full_name;

CREATE OR REPLACE FUNCTION calculate_paid_amount(p_deal_id UUID)
RETURNS NUMERIC
LANGUAGE sql
STABLE
AS $$
    SELECT COALESCE(SUM(amount), 0)
    FROM payments
    WHERE deal_id = p_deal_id;
$$;

CREATE OR REPLACE FUNCTION search_properties(
    p_query TEXT DEFAULT NULL,
    p_category_id INTEGER DEFAULT NULL,
    p_status_id INTEGER DEFAULT NULL,
    p_district_id INTEGER DEFAULT NULL
)
RETURNS TABLE (
    property_id UUID,
    property_title VARCHAR,
    category_name VARCHAR,
    status_name VARCHAR,
    condition_name VARCHAR,
    district_name VARCHAR,
    owner_name VARCHAR,
    area NUMERIC,
    price NUMERIC,
    address TEXT,
    listed_at DATE
)
LANGUAGE sql
STABLE
AS $$
    SELECT
        catalog.property_id,
        catalog.property_title,
        catalog.category_name,
        catalog.status_name,
        catalog.condition_name,
        catalog.district_name,
        catalog.owner_name,
        catalog.area,
        catalog.price,
        catalog.address,
        catalog.listed_at
    FROM vw_property_catalog catalog
    JOIN properties p ON p.id = catalog.property_id
    WHERE (p_query IS NULL OR lower(catalog.property_title) LIKE lower('%' || p_query || '%') OR lower(catalog.address) LIKE lower('%' || p_query || '%'))
      AND (p_category_id IS NULL OR p.category_id = p_category_id)
      AND (p_status_id IS NULL OR p.status_id = p_status_id)
      AND (p_district_id IS NULL OR p.district_id = p_district_id)
    ORDER BY catalog.listed_at DESC;
$$;

CREATE OR REPLACE FUNCTION manager_sales_report(p_from DATE, p_to DATE)
RETURNS TABLE (
    manager_name VARCHAR,
    deals_count BIGINT,
    total_amount NUMERIC,
    total_commission NUMERIC
)
LANGUAGE sql
STABLE
AS $$
    SELECT
        u.full_name AS manager_name,
        COUNT(d.id) AS deals_count,
        COALESCE(SUM(d.amount), 0) AS total_amount,
        COALESCE(SUM(d.commission), 0) AS total_commission
    FROM app_users u
    LEFT JOIN deals d ON d.manager_id = u.id
    LEFT JOIN deal_statuses ds ON ds.id = d.status_id
    WHERE d.start_date BETWEEN p_from AND p_to
      AND ds.code = 'completed'
    GROUP BY u.full_name
    ORDER BY total_amount DESC;
$$;
