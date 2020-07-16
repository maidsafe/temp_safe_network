
    /// Processes any outstanding network events and returns. Does not block.
    /// Returns whether at least one event was processed.
    pub fn poll(&mut self) -> bool {
        let mut _processed = false;
        loop {
            let mut sel = Select::new();
            let mut r_node = self.routing.borrow_mut();
            r_node.register(&mut sel);
            let routing_event_rx_idx = sel.recv(&self.event_receiver);
            let client_network_rx_idx = sel.recv(&self.client_receiver);
            let command_rx_idx = sel.recv(&self.command_receiver);

            if let Ok(selected_operation) = sel.try_ready() {
                drop(r_node);
                match selected_operation {
                    idx if idx == client_network_rx_idx => {
                        let event = match self.client_receiver.recv() {
                            Ok(ev) => ev,
                            Err(e) => panic!("FIXME: {:?}", e),
                        };
                        self.step_client(event);
                        _processed = true;
                    }
                    idx if idx == routing_event_rx_idx => {
                        let event = match self.event_receiver.recv() {
                            Ok(ev) => ev,
                            Err(e) => panic!("FIXME: {:?}", e),
                        };
                        self.step_routing(event);
                        _processed = true;
                    }
                    idx if idx == command_rx_idx => {
                        let command = match self.command_receiver.recv() {
                            Ok(ev) => ev,
                            Err(e) => panic!("FIXME: {:?}", e),
                        };
                        match command {
                            Command::Shutdown => (),
                        }
                        _processed = true;
                    }
                    idx => {
                        if let Err(err) = self.routing.borrow_mut().handle_selected_operation(idx) {
                            warn!("Could not process operation: {}", err);
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
        _processed
    }