-- 1. Список объектов с полной карточкой
SELECT * FROM vw_property_catalog ORDER BY price DESC;

-- 2. Поиск объектов по строке и району
SELECT * FROM search_properties('Минск', NULL, NULL, (SELECT id FROM districts WHERE name = 'Советский'));

-- 3. Сводка по собственникам
SELECT * FROM vw_owner_portfolio ORDER BY total_portfolio_value DESC;

-- 4. Отчет по клиентским предпочтениям
SELECT * FROM vw_client_preferences ORDER BY budget DESC NULLS LAST;

-- 5. Активные и завершенные сделки
SELECT * FROM vw_active_deals ORDER BY start_date DESC;

-- 6. Продажи и аренда по месяцам
SELECT * FROM vw_sales_summary_by_month ORDER BY period_label DESC;

-- 7. Эффективность менеджеров
SELECT * FROM vw_manager_performance ORDER BY total_amount DESC;

-- 8. Общая сумма поступлений по сделке
SELECT
    d.id,
    p.title,
    calculate_paid_amount(d.id) AS paid_total
FROM deals d
JOIN properties p ON p.id = d.property_id;

-- 9. Отчет по менеджерам за период
SELECT * FROM manager_sales_report(DATE '2026-01-01', DATE '2026-12-31');

-- 10. Количество объектов по статусам и категориям
SELECT
    pc.name AS category_name,
    ps.name AS status_name,
    COUNT(*) AS properties_count,
    AVG(p.price) AS average_price
FROM properties p
JOIN property_categories pc ON pc.id = p.category_id
JOIN property_statuses ps ON ps.id = p.status_id
GROUP BY pc.name, ps.name
ORDER BY pc.name, ps.name;
