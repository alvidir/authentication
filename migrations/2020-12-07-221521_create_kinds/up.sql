CREATE TABLE Kinds (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE
);

INSERT INTO Kinds (id, name) VALUES (0, 'USER');
INSERT INTO Kinds (id, name) VALUES (1, 'APP');