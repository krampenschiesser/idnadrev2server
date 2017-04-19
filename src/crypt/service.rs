
enum Command {
    CreateRepository{id: uuid, name: name, },
}

enum Response {
    CreatedRepository(uuid),
}