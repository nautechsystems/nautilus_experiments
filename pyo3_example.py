from nautilus_experiments import (
    SendEvent,
    ChainEvent,
    MessageBusContext,
    RustMessageHandlerWrapper,
)


class PythonHandler:
    def __init__(self, val, context: MessageBusContext):
        self.val = val
        self.data = []
        self.context = context

    def id(self):
        return self.val

    def send(self, id, val):
        self.context.py_send(id, SendEvent(val))

    def start_chain(self, val):
        self.context.py_chain(self.id(), ChainEvent(self.id(), val))

    def handle(self, event):
        if isinstance(event, SendEvent):
            self.data.append(event.data)
        elif isinstance(event, ChainEvent):
            self.data.append(event.data)

            next = self.id() + 1

            # if next doesn't exist, send to first handler next
            if not self.context.check_handler(next):
                next = 0

            # if next is the starting handler stop chain to prevent borrow mut error
            if next != event.start:
                print(f"Sending chain event to {next}")
                self.context.py_chain(next, event)
        else:
            print(f"Unknown event type {event.__class__.__name__}")


if __name__ == "__main__":
    context = MessageBusContext()
    python_handler = PythonHandler(0, context)
    rust_handler_1 = RustMessageHandlerWrapper(1, context)
    rust_handler_2 = RustMessageHandlerWrapper(2, context)

    context.register_python_handler(python_handler.id(), python_handler)
    context.register_rust_handler(rust_handler_1.id(), rust_handler_1)
    context.register_rust_handler(rust_handler_2.id(), rust_handler_2)

    python_handler.send(1, 49)
    rust_handler_1.send(2, 51)

    assert rust_handler_1.get_data() == [49]
    assert rust_handler_2.get_data() == [51]

    context.py_chain(1, ChainEvent(1, 42))
    assert python_handler.data == [42]
    assert rust_handler_1.get_data() == [49, 42]
    assert rust_handler_2.get_data() == [51, 42]

    python_handler.start_chain(99)
    assert python_handler.data == [42, 99]
    assert rust_handler_1.get_data() == [49, 42, 99]
    assert rust_handler_2.get_data() == [51, 42, 99]
