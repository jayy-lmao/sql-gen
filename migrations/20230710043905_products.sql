-- Migration generated for struct: products
ALTER TABLE Products ALTER COLUMN id TYPE int8, SET NOT NULL;
ALTER TABLE Products ALTER COLUMN label TYPE text, SET NOT NULL;
ALTER TABLE Products ALTER COLUMN description TYPE text, DROP NOT NULL;
ALTER TABLE Products ALTER COLUMN category TYPE int8, DROP NOT NULL;
ALTER TABLE Products DROP COLUMN created_at
