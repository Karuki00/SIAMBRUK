ALTER TABLE product_variants ADD COLUMN inventory_value INTEGER NOT NULL DEFAULT 0 CHECK (inventory_value >= 0);

UPDATE product_variants
SET inventory_value = current_stock * purchase_price
WHERE inventory_value = 0 AND current_stock > 0;
