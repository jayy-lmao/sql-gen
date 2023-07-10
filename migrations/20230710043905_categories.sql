-- Migration generated for struct: categories
ALTER TABLE Categories ALTER COLUMN id TYPE int8, SET NOT NULL;
ALTER TABLE Categories ALTER COLUMN label TYPE text, SET NOT NULL;
ALTER TABLE Categories DROP COLUMN created_at
