-- migrations/update_newsletter_issues_table_with_missing_column.sql

ALTER TABLE newsletter_issues
RENAME COLUMN title_content TO text_content;
