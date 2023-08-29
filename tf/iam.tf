resource "aws_iam_role" "iam_for_lambda" {
  name = "dev-machines-role"
  path = "/service-role/"
  assume_role_policy = jsonencode(
    {
      Version = "2012-10-17",
      Statement = {
        Effect = "Allow",
        Principal = {
          Service = "lambda.amazonaws.com"
        },
        Action = "sts:AssumeRole"
      }
  })
}

resource "aws_iam_policy" "iam_for_lambda" {
  name = "dev-machine-lambda-policy"
  policy = jsonencode({
    Version = "2012-10-17",
    Statement = [
      {
        "Effect" : "Allow",
        # "Principal" : {
        #   "Service" : "events.amazonaws.com"
        # },
        "Action" : "lambda:InvokeFunction",
        "Resource" : aws_lambda_function.data_fetch_lambda.arn,
        "Condition" : {
          "ArnLike" : {
            "AWS:SourceArn" : aws_cloudwatch_event_rule.data_fetch_event.arn
          }
        }
      },
      {
        "Effect" : "Allow",
        "Action" : [
          "ec2:DescribeNetworkInterfaces",
          "ec2:CreateNetworkInterface",
          "ec2:DeleteNetworkInterface",
          "ec2:DescribeInstances",
          "ec2:AttachNetworkInterface"
        ],
        "Resource" : "*"
      },
    ]
  })
}

resource "aws_iam_role_policy_attachment" "iam_for_lambda_attachment" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = aws_iam_policy.iam_for_lambda.arn
}

resource "aws_iam_role_policy_attachment" "lambda_rds_data_policy" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/AmazonRDSDataFullAccess"
}


resource "aws_iam_role_policy_attachment" "lambda_cloudwatch_logs_policy" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/CloudWatchLogsFullAccess"
}

resource "aws_iam_role_policy_attachment" "lambda_vpc_access_policy" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}
