pub mod db {
    use sqlx::FromRow;
    use sqlx::PgPool;

    use crate::model::MetaDataModel::MetaDataDBRecord;
    use crate::model::MetaDataModel::MetaDataKafkaRecord;
    use crate::model::VehicleDataModel::Vehicle;
    use crate::writer::sink::Sink;

    #[derive(Debug, FromRow)]
    pub struct VehicleDB {
        pub id: String,
        pub source: String,
        pub make: String,
        pub model: String,
        pub title: String,
        pub year: i32,
        pub mileage: i32,
        pub engine: String,
        pub gearbox: String,
        pub power_ps: i32,
        pub power_kw: i32,
        pub currency: String,
        pub price: i32,
        pub estimated_price: Option<i32>,
        pub cc: Option<i32>,

        pub url: String,
        pub location: Option<String>,
        pub equipment: Option<String>,
        pub seller_name: Option<String>,
        pub seller_url: Option<String>,

        pub range: Option<String>,
        pub consumption_fuel: Option<String>,
        pub consumption_kw: Option<String>,
        pub co2: Option<i32>,

        pub days_in_sale: Option<i32>,
        pub ranges: Option<String>,
        pub rating: Option<String>,
        pub filter_id: Option<String>,
        pub created_on: Option<chrono::NaiveDate>,
        pub updated_on: Option<chrono::NaiveDate>,
        pub deleted_on: Option<chrono::NaiveDate>,
    }

    impl From<Vehicle> for VehicleDB {
        fn from(vehicle: Vehicle) -> Self {
            VehicleDB {
                id: vehicle.id,
                source: vehicle.source,
                make: vehicle.make,
                model: vehicle.model,
                title: vehicle.title,
                year: vehicle.year as i32,
                mileage: vehicle.mileage as i32,
                engine: vehicle.engine.to_string(),
                gearbox: vehicle.gearbox.to_string(),
                power_ps: vehicle.power_ps as i32,
                power_kw: vehicle.power_kw as i32,
                currency: vehicle.currency.to_string(),
                price: vehicle.price as i32,
                estimated_price: vehicle.estimated_price.map(|p| p as i32),
                cc: vehicle.cc.map(|v| v as i32),
                url: vehicle.url,
                location: vehicle.location,
                equipment: vehicle.equipment,
                seller_name: vehicle.seller_name,
                seller_url: vehicle.seller_url,
                range: vehicle.range.map(|v| v.to_string()),
                consumption_fuel: vehicle.consumption_fuel.map(|v| v.to_string()),
                consumption_kw: vehicle.consumption_kw.map(|v| v.to_string()),
                co2: vehicle.co2.map(|v| v as i32),
                days_in_sale: vehicle.days_in_sale.map(|v| v as i32),
                ranges: vehicle.ranges,
                rating: vehicle.rating,
                filter_id: vehicle.filter_id,
                created_on: None,
                updated_on: None,
                deleted_on: None, // Assuming not deleted
            }
        }
    }

    pub struct DBWriter {
        pool: PgPool,
    }

    impl DBWriter {
        pub async fn new() -> Self {
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment");
            let pool = PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to the database");
            DBWriter { pool }
        }
    }

    #[async_trait::async_trait]
    impl Sink<Vehicle> for DBWriter {
        async fn write(&self, vehicle: Vehicle) -> Result<(), String> {
            log::info!("Writing vehicle to database: {:?}", vehicle);
            let db_vehicle = VehicleDB::from(vehicle); // имплементирай From

            let x = sqlx::query!(
                r#"
            INSERT INTO vehicles (
                id, source, make, model, title, year, mileage, engine, gearbox,
                power_ps, power_kw, currency, price, estimated_price, cc,
                url, location, equipment, seller_name, seller_url,
                range, consumption_fuel, consumption_kw, co2,
                days_in_sale, ranges, rating,filter_id
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20,
                $21, $22, $23, $24,
                $25, $26, $27, $28
            )
            ON CONFLICT(id, source) DO UPDATE SET
                updated_on = CURRENT_DATE,
                price = EXCLUDED.price,
                estimated_price = EXCLUDED.estimated_price,
                mileage = EXCLUDED.mileage,
                engine = EXCLUDED.engine,
                gearbox = EXCLUDED.gearbox,
                power_ps = EXCLUDED.power_ps,
                power_kw = EXCLUDED.power_kw,
                currency = EXCLUDED.currency,
                year = EXCLUDED.year,
                url = EXCLUDED.url,
                cc = EXCLUDED.cc,
                make = EXCLUDED.make,
                model = EXCLUDED.model,
                title = EXCLUDED.title,
                location = EXCLUDED.location,
                equipment = EXCLUDED.equipment,
                seller_name = EXCLUDED.seller_name,
                seller_url = EXCLUDED.seller_url,
                range = EXCLUDED.range,
                consumption_fuel = EXCLUDED.consumption_fuel,
                consumption_kw = EXCLUDED.consumption_kw,
                co2 = EXCLUDED.co2,
                days_in_sale = EXCLUDED.days_in_sale,
                ranges = EXCLUDED.ranges,
                rating = EXCLUDED.rating,
                filter_id = EXCLUDED.filter_id
            "#,
                db_vehicle.id,
                db_vehicle.source,
                db_vehicle.make,
                db_vehicle.model,
                db_vehicle.title,
                db_vehicle.year,
                db_vehicle.mileage,
                db_vehicle.engine as _,
                db_vehicle.gearbox as _,
                db_vehicle.power_ps,
                db_vehicle.power_kw,
                db_vehicle.currency as _,
                db_vehicle.price,
                db_vehicle.estimated_price,
                db_vehicle.cc,
                db_vehicle.url,
                db_vehicle.location,
                db_vehicle.equipment,
                db_vehicle.seller_name,
                db_vehicle.seller_url,
                db_vehicle.range,
                db_vehicle.consumption_fuel,
                db_vehicle.consumption_kw,
                db_vehicle.co2,
                db_vehicle.days_in_sale,
                db_vehicle.ranges,
                db_vehicle.rating,
                db_vehicle.filter_id,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                log::info!("ERROR: {:?}", e);
                format!("DB insert error: {e}")
            })?;
            log::info!("Vehicle written to database: {:?}", x);
            Ok(())
        }

        async fn flush(&self) -> Result<(), String> {
            Ok(()) // no-op
        }
    }

    pub struct MetaDBWriter {
        pool: PgPool,
    }

    impl MetaDBWriter {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }

        /// Записва или актуализира meta записа в таблицата `metadata`.
        pub async fn write(&self, record: MetaDataKafkaRecord) -> Result<(), sqlx::Error> {
            let db_record: MetaDataDBRecord = MetaDataKafkaRecord::from(record).into();

            sqlx::query!(
                r#"
            INSERT INTO metadata (
                filter_id, source, flow, page_type, url, filters, equipment, last_run_on
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (filter_id)
            DO UPDATE SET
                source = EXCLUDED.source,
                flow = EXCLUDED.flow,
                page_type = EXCLUDED.page_type,
                url = EXCLUDED.url,
                filters = EXCLUDED.filters,
                equipment = EXCLUDED.equipment,
                last_run_on = EXCLUDED.last_run_on
            "#,
                db_record.filter_id,
                db_record.source,
                db_record.flow,
                db_record.page_type,
                db_record.url,
                db_record.filters,
                &db_record.equipment,
                db_record.last_run_on,
            )
            .execute(&self.pool)
            .await?;

            Ok(())
        }
    }
}
