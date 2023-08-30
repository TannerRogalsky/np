resource "aws_cloudwatch_event_rule" "data_fetch_event" {
  name        = "dev-machines-data-fetch-cron"
  description = "Runs the dev machine shutdown reminder lambda."

  schedule_expression = "rate(15 minutes)"
}

resource "aws_cloudwatch_event_target" "data_fetch_event" {
  rule = aws_cloudwatch_event_rule.data_fetch_event.name
  arn  = aws_lambda_function.data_fetch_lambda.arn
  input = jsonencode({
    "game_id" : 4725907836895232
  })
}
