resource "aws_lambda_function" "data_fetch_lambda" {
  function_name = "dev-machine-state-tracking"
  role          = aws_iam_role.iam_for_lambda.arn

  filename         = "${path.module}/../target/lambda/data-fetch/bootstrap.zip"
  source_code_hash = filebase64sha256("${path.module}/../target/lambda/data-fetch/bootstrap.zip")

  handler = "hello.handler"
  runtime = "provided.al2"
  timeout = 3

  vpc_config {
    subnet_ids = [
      # data.aws_subnet.dev_machine_subnet.id,
      data.aws_subnet.dev_machine_private_subnet.id
    ]
    security_group_ids = [
      data.aws_security_group.tanner_dev.id,
      data.aws_security_group.office_ssh.id,
    ]
  }

  environment {
    variables = {
      "RUST_LOG"     = "debug"
      "DATABASE_URL" = "postgres://${aws_rds_cluster.dev_machine_db.master_username}:${local.db_password}@${aws_rds_cluster.dev_machine_db.endpoint}/${aws_rds_cluster.dev_machine_db.database_name}"
    }
  }
}

