data "aws_rds_engine_version" "postgresql" {
  engine  = "aurora-postgresql"
  version = "14.7"
}

resource "aws_rds_cluster" "dev_machine_db" {
  cluster_identifier = "dev-machine-db"
  engine             = data.aws_rds_engine_version.postgresql.engine
  engine_version     = data.aws_rds_engine_version.postgresql.version
  engine_mode        = "provisioned"

  database_name   = "np"
  master_username = "dev_machine_user"
  master_password = local.db_password

  copy_tags_to_snapshot = true
  skip_final_snapshot   = true

  serverlessv2_scaling_configuration {
    max_capacity = 2.0
    min_capacity = 0.5
  }
}

resource "aws_rds_cluster_instance" "dev_machine_db" {
  cluster_identifier = aws_rds_cluster.dev_machine_db.id
  instance_class     = "db.serverless"
  engine             = aws_rds_cluster.dev_machine_db.engine
  engine_version     = aws_rds_cluster.dev_machine_db.engine_version
}
