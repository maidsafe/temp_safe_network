reduce inputs as $item ([]; . + [$item])
| map(select(.fields.message))
| map(select(.fields.message | (contains("ConnectionOpened") or contains("ConnectionClosed"))))
| map({
    timestamp,
    event: .fields.message,
    conn_id: .fields.connection_id,
    src_addr: .fields.src,
    trace: .spans | map("\(.name)\(if .command then " (\(.command))" else "" end)\(if .recipients then " (\(.recipients))" else "" end)") | join(" -> ")
  })
| sort_by(.timestamp)
