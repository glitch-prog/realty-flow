BEGIN;

UPDATE clients
SET preferred_category_id = CASE preferred_category_id
    WHEN 4 THEN 1
    WHEN 5 THEN 2
    WHEN 6 THEN 3
    ELSE preferred_category_id
END
WHERE preferred_category_id IN (4, 5, 6);

UPDATE properties
SET category_id = CASE category_id
        WHEN 4 THEN 1
        WHEN 5 THEN 2
        WHEN 6 THEN 3
        ELSE category_id
    END,
    condition_id = CASE condition_id
        WHEN 4 THEN 1
        WHEN 5 THEN 2
        WHEN 6 THEN 3
        ELSE condition_id
    END,
    district_id = CASE district_id
        WHEN 5 THEN 1
        WHEN 6 THEN 2
        WHEN 7 THEN 3
        WHEN 8 THEN 4
        ELSE district_id
    END
WHERE category_id IN (4, 5, 6)
   OR condition_id IN (4, 5, 6)
   OR district_id IN (5, 6, 7, 8);

DELETE FROM clients c
USING clients canonical
WHERE c.phone = canonical.phone
  AND c.id <> canonical.id
  AND canonical.full_name = 'ООО Горизонт Бизнес'
  AND c.full_name = 'РћРћРћ Р“РѕСЂРёР·РѕРЅС‚ Р‘РёР·РЅРµСЃ'
  AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.client_id = c.id);

DELETE FROM property_categories WHERE id IN (4, 5, 6);
DELETE FROM property_conditions WHERE id IN (4, 5, 6);
DELETE FROM districts WHERE id IN (5, 6, 7, 8);

UPDATE property_categories
SET name = CASE id
    WHEN 1 THEN 'Квартира'
    WHEN 2 THEN 'Дом'
    WHEN 3 THEN 'Коммерческая недвижимость'
END
WHERE id IN (1, 2, 3);

UPDATE property_conditions
SET name = CASE id
    WHEN 1 THEN 'Новое'
    WHEN 2 THEN 'Хорошее'
    WHEN 3 THEN 'Требует ремонта'
END
WHERE id IN (1, 2, 3);

UPDATE districts
SET city = 'Минск',
    name = CASE id
        WHEN 1 THEN 'Центральный'
        WHEN 2 THEN 'Советский'
        WHEN 3 THEN 'Фрунзенский'
        WHEN 4 THEN 'Московский'
    END
WHERE id IN (1, 2, 3, 4);

UPDATE property_statuses
SET name = CASE code
    WHEN 'free' THEN 'Свободно'
    WHEN 'in_deal' THEN 'В сделке'
    WHEN 'sold' THEN 'Продано'
    WHEN 'rented' THEN 'Сдано'
    WHEN 'archived' THEN 'Архив'
END;

UPDATE client_types
SET name = CASE code
    WHEN 'buyer' THEN 'Покупатель'
    WHEN 'tenant' THEN 'Арендатор'
END;

UPDATE deal_types
SET name = CASE code
    WHEN 'sale' THEN 'Купля-продажа'
    WHEN 'rent' THEN 'Аренда'
END;

UPDATE deal_statuses
SET name = CASE code
    WHEN 'registered' THEN 'Зарегистрирована'
    WHEN 'in_progress' THEN 'В сопровождении'
    WHEN 'completed' THEN 'Завершена'
    WHEN 'cancelled' THEN 'Отменена'
END;

COMMIT;
