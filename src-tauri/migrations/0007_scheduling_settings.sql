ALTER TABLE scheduler_configurations
ADD COLUMN maximum_interval_days INTEGER NOT NULL DEFAULT 36500
    CHECK (maximum_interval_days BETWEEN 1 AND 36500);
