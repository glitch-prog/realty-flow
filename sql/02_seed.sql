INSERT INTO user_roles (code, name, description)
VALUES
    ('admin', 'Администратор', 'Полный доступ к системе и журналу аудита'),
    ('realtor', 'Риелтор', 'Работа с объектами, клиентами, собственниками и сделками'),
    ('guest', 'Гость', 'Просмотр данных и отчетов без права изменения'),
    ('manager', 'Менеджер', 'Совместимость с предыдущей версией ролей проекта'),
    ('analyst', 'Аналитик', 'Совместимость с предыдущей версией ролей проекта')
ON CONFLICT (code) DO NOTHING;

INSERT INTO app_users (role_id, username, password_plain, full_name, email)
SELECT r.id, 'admin', encode(digest('admin123', 'sha256'), 'hex'), 'Администратор системы', 'admin@realtyflow.local'
FROM user_roles r
WHERE r.code = 'admin'
ON CONFLICT (username) DO NOTHING;

INSERT INTO app_users (role_id, username, password_plain, full_name, email)
SELECT r.id, 'realtor', encode(digest('realtor123', 'sha256'), 'hex'), 'Риелтор агентства', 'realtor@realtyflow.local'
FROM user_roles r
WHERE r.code = 'realtor'
ON CONFLICT (username) DO NOTHING;

INSERT INTO app_users (role_id, username, password_plain, full_name, email)
SELECT r.id, 'guest', encode(digest('guest123', 'sha256'), 'hex'), 'Гостевой пользователь', 'guest@realtyflow.local'
FROM user_roles r
WHERE r.code = 'guest'
ON CONFLICT (username) DO NOTHING;

INSERT INTO app_users (role_id, username, password_plain, full_name, email)
SELECT r.id, 'manager', encode(digest('manager123', 'sha256'), 'hex'), 'Менеджер по сделкам', 'manager@realtyflow.local'
FROM user_roles r
WHERE r.code = 'manager'
ON CONFLICT (username) DO NOTHING;

INSERT INTO app_users (role_id, username, password_plain, full_name, email)
SELECT r.id, 'analyst', encode(digest('analyst123', 'sha256'), 'hex'), 'Аналитик агентства', 'analyst@realtyflow.local'
FROM user_roles r
WHERE r.code = 'analyst'
ON CONFLICT (username) DO NOTHING;

INSERT INTO property_categories (name, description)
VALUES
    ('Квартира', 'Жилая квартира'),
    ('Дом', 'Частный дом или коттедж'),
    ('Коммерческая недвижимость', 'Офисы, склады и торговые помещения')
ON CONFLICT (name) DO NOTHING;

INSERT INTO property_statuses (code, name)
VALUES
    ('free', 'Свободно'),
    ('in_deal', 'В сделке'),
    ('sold', 'Продано'),
    ('rented', 'Сдано'),
    ('archived', 'Архив')
ON CONFLICT (code) DO NOTHING;

INSERT INTO property_conditions (name)
VALUES
    ('Новое'),
    ('Хорошее'),
    ('Требует ремонта')
ON CONFLICT (name) DO NOTHING;

INSERT INTO districts (city, name)
VALUES
    ('Минск', 'Центральный'),
    ('Минск', 'Советский'),
    ('Минск', 'Фрунзенский'),
    ('Минск', 'Московский')
ON CONFLICT (name) DO NOTHING;

INSERT INTO client_types (code, name)
VALUES
    ('buyer', 'Покупатель'),
    ('tenant', 'Арендатор')
ON CONFLICT (code) DO NOTHING;

INSERT INTO deal_types (code, name)
VALUES
    ('sale', 'Купля-продажа'),
    ('rent', 'Аренда')
ON CONFLICT (code) DO NOTHING;

INSERT INTO deal_statuses (code, name)
VALUES
    ('registered', 'Зарегистрирована'),
    ('in_progress', 'В сопровождении'),
    ('completed', 'Завершена'),
    ('cancelled', 'Отменена')
ON CONFLICT (code) DO NOTHING;

INSERT INTO owners (full_name, phone, email, passport_no, notes)
VALUES
    ('Ковалев Сергей Викторович', '+375291111111', 'kovalev@example.com', 'MP1234567', 'Собственник нескольких квартир'),
    ('Морозова Анна Игоревна', '+375292222222', 'morozova@example.com', 'MP7654321', 'Заинтересована в долгосрочной аренде'),
    ('Иванов Павел Сергеевич', '+375293333333', 'ivanov@example.com', 'HB4567123', 'Коммерческая недвижимость')
ON CONFLICT (phone) DO NOTHING;

INSERT INTO clients (client_type_id, full_name, phone, email, budget, preferred_category_id, notes)
SELECT
    ct.id,
    'Петров Максим Андреевич',
    '+375294444444',
    'petrov@example.com',
    135000.00,
    pc.id,
    'Ищет квартиру в центре'
FROM client_types ct
JOIN property_categories pc ON pc.name = 'Квартира'
WHERE ct.code = 'buyer'
ON CONFLICT ON CONSTRAINT uq_clients_name_phone DO NOTHING;

INSERT INTO clients (client_type_id, full_name, phone, email, budget, preferred_category_id, notes)
SELECT
    ct.id,
    'Орлова Екатерина Дмитриевна',
    '+375295555555',
    'orlova@example.com',
    1800.00,
    pc.id,
    'Интересует аренда дома'
FROM client_types ct
JOIN property_categories pc ON pc.name = 'Дом'
WHERE ct.code = 'tenant'
ON CONFLICT ON CONSTRAINT uq_clients_name_phone DO NOTHING;

INSERT INTO clients (client_type_id, full_name, phone, email, budget, preferred_category_id, notes)
SELECT
    ct.id,
    'ООО Горизонт Бизнес',
    '+375296666666',
    'office@example.com',
    350000.00,
    pc.id,
    'Подбор коммерческого помещения под офис'
FROM client_types ct
JOIN property_categories pc ON pc.name = 'Коммерческая недвижимость'
WHERE ct.code = 'buyer'
ON CONFLICT ON CONSTRAINT uq_clients_name_phone DO NOTHING;

INSERT INTO properties (
    owner_id,
    category_id,
    status_id,
    condition_id,
    district_id,
    title,
    area,
    price,
    address,
    description,
    listed_at
)
SELECT
    o.id,
    pc.id,
    ps.id,
    cond.id,
    d.id,
    'Трехкомнатная квартира у Комаровки',
    82.50,
    128000.00,
    'г. Минск, ул. Куйбышева, 48',
    'Квартира с ремонтом и мебелью',
    DATE '2026-01-15'
FROM owners o
JOIN property_categories pc ON pc.name = 'Квартира'
JOIN property_statuses ps ON ps.code = 'free'
JOIN property_conditions cond ON cond.name = 'Хорошее'
JOIN districts d ON d.name = 'Советский'
WHERE o.phone = '+375291111111'
ON CONFLICT ON CONSTRAINT uq_properties_owner_address DO NOTHING;

INSERT INTO properties (
    owner_id,
    category_id,
    status_id,
    condition_id,
    district_id,
    title,
    area,
    price,
    address,
    description,
    listed_at
)
SELECT
    o.id,
    pc.id,
    ps.id,
    cond.id,
    d.id,
    'Коттедж с участком',
    164.00,
    2100.00,
    'г. Минск, пер. Озерный, 7',
    'Дом для аренды, меблирован',
    DATE '2026-02-01'
FROM owners o
JOIN property_categories pc ON pc.name = 'Дом'
JOIN property_statuses ps ON ps.code = 'free'
JOIN property_conditions cond ON cond.name = 'Хорошее'
JOIN districts d ON d.name = 'Центральный'
WHERE o.phone = '+375292222222'
ON CONFLICT ON CONSTRAINT uq_properties_owner_address DO NOTHING;

INSERT INTO properties (
    owner_id,
    category_id,
    status_id,
    condition_id,
    district_id,
    title,
    area,
    price,
    address,
    description,
    listed_at
)
SELECT
    o.id,
    pc.id,
    ps.id,
    cond.id,
    d.id,
    'Офис open-space 240 м2',
    240.00,
    355000.00,
    'г. Минск, пр-т Дзержинского, 90',
    'Коммерческое помещение с отдельным входом',
    DATE '2026-02-10'
FROM owners o
JOIN property_categories pc ON pc.name = 'Коммерческая недвижимость'
JOIN property_statuses ps ON ps.code = 'free'
JOIN property_conditions cond ON cond.name = 'Новое'
JOIN districts d ON d.name = 'Московский'
WHERE o.phone = '+375293333333'
ON CONFLICT ON CONSTRAINT uq_properties_owner_address DO NOTHING;

INSERT INTO deals (
    property_id,
    client_id,
    manager_id,
    deal_type_id,
    status_id,
    amount,
    commission,
    start_date,
    end_date,
    notes
)
SELECT
    p.id,
    c.id,
    u.id,
    dt.id,
    ds.id,
    128000.00,
    3840.00,
    DATE '2026-02-15',
    DATE '2026-02-20',
    'Сделка купли-продажи с ипотекой'
FROM properties p
JOIN clients c ON c.phone = '+375294444444'
JOIN app_users u ON u.username IN ('realtor', 'manager')
JOIN deal_types dt ON dt.code = 'sale'
JOIN deal_statuses ds ON ds.code = 'completed'
WHERE p.address = 'г. Минск, ул. Куйбышева, 48'
  AND NOT EXISTS (
      SELECT 1
      FROM deals d
      WHERE d.property_id = p.id
        AND d.client_id = c.id
        AND d.start_date = DATE '2026-02-15'
  )
ORDER BY u.username = 'realtor' DESC
LIMIT 1;

INSERT INTO deals (
    property_id,
    client_id,
    manager_id,
    deal_type_id,
    status_id,
    amount,
    commission,
    start_date,
    end_date,
    notes
)
SELECT
    p.id,
    c.id,
    u.id,
    dt.id,
    ds.id,
    2100.00,
    210.00,
    DATE '2026-03-01',
    DATE '2026-12-31',
    'Долгосрочная аренда дома'
FROM properties p
JOIN clients c ON c.phone = '+375295555555'
JOIN app_users u ON u.username IN ('realtor', 'manager')
JOIN deal_types dt ON dt.code = 'rent'
JOIN deal_statuses ds ON ds.code = 'in_progress'
WHERE p.address = 'г. Минск, пер. Озерный, 7'
  AND NOT EXISTS (
      SELECT 1
      FROM deals d
      WHERE d.property_id = p.id
        AND d.client_id = c.id
        AND d.start_date = DATE '2026-03-01'
  )
ORDER BY u.username = 'realtor' DESC
LIMIT 1;

INSERT INTO payments (deal_id, paid_at, amount, payment_method, comment)
SELECT
    d.id,
    DATE '2026-02-20',
    128000.00,
    'Банковский перевод',
    'Оплата по договору купли-продажи'
FROM deals d
JOIN properties p ON p.id = d.property_id
WHERE p.address = 'г. Минск, ул. Куйбышева, 48'
  AND d.start_date = DATE '2026-02-15'
  AND NOT EXISTS (
      SELECT 1
      FROM payments pay
      WHERE pay.deal_id = d.id
        AND pay.paid_at = DATE '2026-02-20'
  );

INSERT INTO payments (deal_id, paid_at, amount, payment_method, comment)
SELECT
    d.id,
    DATE '2026-03-05',
    2100.00,
    'Наличные',
    'Первый арендный платеж'
FROM deals d
JOIN properties p ON p.id = d.property_id
WHERE p.address = 'г. Минск, пер. Озерный, 7'
  AND d.start_date = DATE '2026-03-01'
  AND NOT EXISTS (
      SELECT 1
      FROM payments pay
      WHERE pay.deal_id = d.id
        AND pay.paid_at = DATE '2026-03-05'
  );
