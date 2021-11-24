reduce inputs as $item ([]; . + [$item + {input_filename: input_filename}])
| map(select(.fields.message))
| map(select(.fields.message | (contains("ConnectionOpened") or contains("ConnectionClosed"))))
| map({
    input_filename,
    timestamp,
    event: .fields.message,
    conn_id: .fields.connection_id,
    src_addr: .fields.src,
    trace: .spans | map(
        . as $span
        | {}
        | if $span.command then . + {command: $span.command} else . end
        | if $span.recipients then . + {recipients: $span.recipients} else . end
        | if $span.wire_msg then . + {wire_msg: $span.wire_msg} else . end
        | if length > 0 then {"\($span.name)": .} else $span.name end
    )
  })
| sort_by(.timestamp)
