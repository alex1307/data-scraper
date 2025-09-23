#!/bin/bash

set -e

# CONFIGURATION
DB_NAME="vehicles"
DB_USER="admin"  # change if different
DB_PASSWORD="1234"  # change if different
CONTAINER_NAME="postgres-server"  # change if different
TABLE_NAME="vehicles"
SNAPSHOT="vehicles_snapshot"
# Expect the path to CSV as the first argument
CSV_FILE=$1
FILE_DATE=$(echo "$CSV_FILE" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}')

echo "Date part from filename: $FILE_DATE"

if [[ ! -f "$CSV_FILE" ]]; then
  echo "CSV file '$CSV_FILE' not found."
  exit 1
fi

docker exec -i $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME -c "
DROP TABLE IF EXISTS $SNAPSHOT;
CREATE TABLE $SNAPSHOT AS SELECT * FROM vehicles WITH NO DATA;
CREATE SEQUENCE IF NOT EXISTS vehicles_snapshot_vehicle_id_seq;
ALTER TABLE $SNAPSHOT ALTER COLUMN vehicle_id SET DEFAULT nextval('vehicles_snapshot_vehicle_id_seq');
"



# Extract the date from filename (e.g. vehicles-2025-04-11.csv)
DATE_PART=$(basename "$CSV_FILE" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}')
if [[ -z "$DATE_PART" ]]; then
  echo "Date not found in filename. Expected format: vehicles-YYYY-MM-DD.csv"
  exit 1
fi

# Optional: remove header if Postgres table already has it
TEMP_FILE="/tmp/import_vehicles_$DATE_PART.csv"
tail -n +2 "$CSV_FILE" > "$TEMP_FILE"

# Import using psql inside Docker
docker exec -i $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME -c "\
    COPY $SNAPSHOT (
        id, source, make, model, title, year, mileage, engine, gearbox, power_ps, power_kw,
        currency, price, estimated_price, cc, url, location, equipment, seller_name, seller_url,
        range, consumption_fuel, consumption_kw, co2, days_in_sale, ranges, rating
    )
    FROM STDIN
    DELIMITER ';'
    CSV;
" < "$TEMP_FILE"

# Import using psql inside Docker
docker exec -i $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME -c "
UPDATE $SNAPSHOT
SET created_on = '$FILE_DATE'
WHERE created_on IS NULL"


# Initialize counters
INSERT_COUNT=0
UPDATE_COUNT=0
DELETE_COUNT=0

# Total rows in the snapshot (from the CSV)
TOTAL_COUNT=$(docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*) FROM $SNAPSHOT;
" | xargs)
echo "Total rows in CSV snapshot: $TOTAL_COUNT"

# Insert new records
INSERT_COUNT=$(docker exec -i $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME -t -c "
INSERT INTO $TABLE_NAME (
  id, source, make, model, title, year, mileage, engine, gearbox,
  power_ps, power_kw, currency, price, estimated_price, cc, url,
  location, equipment, seller_name, seller_url, range, consumption_fuel,
  consumption_kw, co2, days_in_sale, ranges, rating, created_on
)
SELECT
  s.id, s.source, s.make, s.model, s.title, s.year, s.mileage, s.engine, s.gearbox,
  s.power_ps, s.power_kw, s.currency, s.price, s.estimated_price, s.cc, s.url,
  s.location, s.equipment, s.seller_name, s.seller_url, s.range, s.consumption_fuel,
  s.consumption_kw, s.co2, s.days_in_sale, s.ranges, s.rating, DATE '$FILE_DATE'
FROM $SNAPSHOT s
LEFT JOIN vehicles v ON s.id = v.id AND s.source = v.source
WHERE v.id IS NULL
RETURNING 1;" | wc -l)
echo "Inserted: $INSERT_COUNT"

echo "✅ Imported '$CSV_FILE' into '$DB_NAME.$TABLE_NAME'"

# Update existing records with updated_on
UPDATE_COUNT=$(docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -t -c  "
UPDATE vehicles SET
  (make, model, title, year, mileage, engine, gearbox, power_ps, power_kw, currency, price,
   estimated_price, cc, url, location, equipment, seller_name, seller_url, range, consumption_fuel,
   consumption_kw, co2, days_in_sale, ranges, rating, updated_on) =
  (v.make, v.model, v.title, v.year, v.mileage, v.engine, v.gearbox, v.power_ps, v.power_kw, v.currency, v.price,
   v.estimated_price, v.cc, v.url, v.location, v.equipment, v.seller_name, v.seller_url, v.range, v.consumption_fuel,
   v.consumption_kw, v.co2, v.days_in_sale, v.ranges, v.rating, DATE '$FILE_DATE')
FROM $SNAPSHOT v
WHERE vehicles.id = v.id AND vehicles.source = v.source
RETURNING 1;
" | wc -l)
echo "Updated: $UPDATE_COUNT"

# Mark deleted rows
DELETE_COUNT=$(docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -t -c  "
UPDATE vehicles SET deleted_on = DATE '$FILE_DATE'
WHERE (id, source) NOT IN (
  SELECT id, source FROM $SNAPSHOT
)
AND deleted_on IS NULL
RETURNING 1;
" | wc -l)
echo "Deleted: $DELETE_COUNT"

echo "Summary: total=$TOTAL_COUNT, inserted=$INSERT_COUNT, updated=$UPDATE_COUNT, deleted=$DELETE_COUNT"