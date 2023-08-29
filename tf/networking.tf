data "aws_vpc" "dev_machine_vpc" {
  id = "vpc-7ee2ce04"
}

data "aws_subnet" "dev_machine_private_subnet" {
  id = "subnet-010a110375b3a7de7"
}

data "aws_subnet" "dev_machine_subnet" {
  id = "subnet-e96ea4e7"
}

data "aws_security_group" "tanner_dev" {
  id = "sg-093590d9cbf06ea1d"
}

data "aws_security_group" "office_ssh" {
  id = "sg-0a51d9587e83dff0a"
}

resource "aws_security_group" "rds_lambda" {
  name        = "rds-lambda-1"
  description = "Security group attached to dev-machine-db to allow Lambda function with specific security groups attached to connect to the RDS database. Modification could lead to connection loss."
  vpc_id      = data.aws_vpc.dev_machine_vpc.id
  ingress = [
    {
      cidr_blocks      = ["172.31.64.0/20"] // dev subnet
      description      = "Rule to allow connections from Lambda function with sg-0fd95c14121b0652e attached"
      from_port        = 5432
      ipv6_cidr_blocks = []
      prefix_list_ids  = []
      protocol         = "tcp"
      security_groups = [
        # aws_security_group.lambda_rds.id,
        # "sg-0fd95c14121b0652e"
      ]
      self    = false
      to_port = 5432
    },
  ]
  egress = []
}
