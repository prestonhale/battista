
let discoveredRooms = [];
let width;
let height;
let roomSize = 12;
let wallWidth = roomSize/10;
let state = {player_position: {x: 0, y: 0}};
let user_id;
let other_players = {};

let edgeTypes = {
    WALL: "Wall",
    PASSAGE: "Passage",
}

let cellTypes = {
    SOIL: "Soil",
    PLANT: "Plant",
    FLOWER: "Flower",
}

let messageTypes = {
    MOVE_PLAYER: "MovePlayer",
    FULL_MAP: "FullMap"
}

function main(){
    document.getElementById('registration').addEventListener('submit', e => {
        let username = document.getElementById('username').value;
        console.log(username);
        submit(username);
        e.preventDefault();
    })
}

function submit(username){
    registerUser(username)
    .then(data => {
        height = data.height;
        width = data.width;

        // let canvas = document.getElementById('game');
        // canvas.height = height * roomSize;
        // canvas.width = width * roomSize;

        console.log(`User id 1 registered with websocket uuid: ${data.id}`);

        // RENDERER =============================================
        const renderer = {
            char_width: 10 * 3,
            char_height: 12 * 3,
            canvas: document.getElementById('game'),
            setFont: setFont,
            start: start,
            drawBackground: drawBackground,
            drawPlayer: drawPlayer,
            drawOtherPlayers: drawOtherPlayers,
            drawRoom: drawRoom,
            drawWalls: drawWalls,
        }

        function start() {
            this.canvas.height = height * this.char_height;
            this.canvas.width = width * this.char_width;
        }

        function setFont() {
            this.font = new Image(320, 96)
            this.font.src = "font_10_12.png"
        }

        function drawBackground(canvas){
            ctx.beginPath();
            ctx.fillStyle = "black";
            ctx.fillRect(
                0, 0,
                width * this.char_width, 
                height * this.char_height,
            );
            ctx.stroke();
        }

        function drawRoom(cell){
            let coords = getCoordsFromIndex(cell.index);
            let x = coords[0];
            let y = coords[1];

            let leftX = x * this.char_width;
            let topY = y * this.char_height;

            ctx.beginPath();
            if (cell.cell_type == cellTypes.SOIL){
                image_coords = {x: 16, y: 5};
            } else if (cell.cell_type == cellTypes.PLANT) {
                image_coords = {x: 27, y: 7};
            } else if (cell.cell_type == cellTypes.FLOWER) {
                image_coords = {x: 23, y: 4};
            }
            ctx.imageSmoothingEnabled = false;
            ctx.drawImage(
                this.font,
                image_coords.x * 10, image_coords.y * 12,
                10, 12,
                leftX, topY,
                this.char_width, this.char_height
            )
        }

        function drawWalls(cell){
            let coords = getCoordsFromIndex(cell.index);
            let x = coords[0];
            let y = coords[1];

            let leftX = x * this.char_width;
            let rightX = (x * this.char_width) + this.char_width;
            let topY = y * this.char_height;
            let bottomY = (y * this.char_height) + this.char_height;

            ctx.lineWidth=wallWidth;

            ctx.beginPath()
            ctx.strokeStyle = "brown";
            if (cell.edges.North == edgeTypes.WALL) {
                ctx.moveTo(leftX, topY);
                ctx.lineTo(rightX, topY);
            };

            if (cell.edges.East == edgeTypes.WALL) {
                ctx.moveTo(rightX, topY);
                ctx.lineTo(rightX, bottomY);
            };

            if (cell.edges.South == edgeTypes.WALL) {
                ctx.moveTo(rightX, bottomY);
                ctx.lineTo(leftX, bottomY);
            };

            if (cell.edges.West == edgeTypes.WALL) {
                ctx.moveTo(leftX, bottomY);
                ctx.lineTo(leftX, topY);
            };

            ctx.stroke();
        }


        function drawPlayer(player_position, player_direction) {
            // console.log("Rendering player with pos: " + player_position.x + "," + player_position.y + " and direction: " + player_direction);
            let x = player_position.x;
            let y = player_position.y;

            let leftX = x * this.char_width;
            let rightX = (x * this.char_width) + this.char_width;
            let topY = y * this.char_height;
            let bottomY = (y * this.char_height) + this.char_height;

            // adjust
            let shrink = this.char_height/10;
            player_size = this.char_height - (shrink*2);
            playerLeftX = leftX + shrink;
            playerTopY = topY + shrink;
            playerRightX = rightX - shrink;
            playerBottomY = bottomY - shrink;

            ctx.beginPath();
            ctx.fillStyle = "blue";
            // Body
            ctx.fillRect(
                playerLeftX,
                playerTopY,
                player_size,
                player_size,
            );
            ctx.stroke();

            // Nose/DirectionIndicator
            switch (player_direction) {
                case "North": 
                    ctx.fillStyle="purple";
                    ctx.fillRect(
                        playerLeftX + (player_size/3),
                        topY,
                        player_size/3,
                        player_size/3,
                    )
                    break;
                case "East": 
                    ctx.fillStyle="purple";
                    ctx.fillRect(
                        rightX - (player_size/3),
                        playerTopY + (player_size/3),
                        player_size/3,
                        player_size/3,
                    )
                    break;
                case "South": 
                    ctx.fillStyle="purple";
                    ctx.fillRect(
                        playerLeftX + (player_size/3),
                        bottomY - (player_size/3),
                        player_size/3,
                        player_size/3,
                    )
                    break;
                case "West": 
                    ctx.fillStyle="purple";
                    ctx.fillRect(
                        leftX,
                        playerTopY + (player_size/3),
                        player_size/3,
                        player_size/3,
                    )
                    break;
            }
            ctx.stroke();

        }

        function drawOtherPlayers() {
            for (const property in other_players) {
                let x = other_players[property].x;
                let y = other_players[property].y;

                let leftX = x * roomSize;
                let rightX = (x * roomSize) + roomSize;
                let topY = y * roomSize;
                let bottomY = (y * roomSize) + roomSize;

                // adjust
                let shrink = roomSize/10
                leftX += shrink;
                topY += shrink;
                color = "black";

                ctx.beginPath();
                ctx.fillStyle = color;
                ctx.fillRect(
                    leftX,
                    topY,
                    roomSize - shrink * 2,
                    roomSize - shrink * 2,
                );
                ctx.stroke();
            }
        }

        // =====================================================
        var Game = {};
        Game.fps = 30;

        Game.renderer = renderer;
        Game.renderer.setFont();
        Game.renderer.start();

        Game.new_input_this_frame = false;
        Game.socket = new WebSocket(data.url);
        Game.state = {}
        Game.state.player_position = data.player_position;
        Game.state.player_direction = data.player_direction;
        Game.state.discoveredRooms = []
        for (const property in data.explored_cells){
            Game.state.discoveredRooms[property] = data.explored_cells[property]
        }
        console.log(Game.state.discoveredRooms);

        // Setup websocket listener
        let new_cells = {};
        let new_position = null;
        Game.socket.addEventListener('message', data =>{
            let msg = data.data;
            msg = JSON.parse(msg);
            if (msg.type == "cell_update"){
                msg.cells.forEach(cell => Game.state.discoveredRooms[cell.index] = cell)
            } else if (msg.type == "player_update") {
                msg.players.forEach(player => new_position=player.coords)
                // Hack
                Game.state.player_position = msg.players[0].coords;
                Game.state.player_direction = msg.players[0].direction;
            }
        })

        Game.socket.addEventListener('open', function (event) {
            var control_map = {
                "Right": "east",
                "ArrowRight": "east",
                "d": "east",

                "Left": "west",
                "ArrowLeft": "west",
                "a": "west",

                "Up": "north",
                "ArrowUp": "north",
                "w": "north",

                "Down": "south",
                "ArrowDown": "south",
                "s": "south",

                " ": "interact"
            }
            
            const curInput = {
                "north": false,
                "east": false,
                "south": false,
                "west": false,
                "interact": false
            };

            const keyDownHandler = (e) => {
                command = control_map[e.key];
                if (command !== undefined && !curInput[command]) {
                    console.log("Key pressed:", e.key);
                    e.preventDefault();

                    curInput[command] = true;
                    Game.new_input_this_frame = true;
                }

            }

            const keyUpHandler = (e) => {
                command = control_map[e.key];
                if (command !== undefined && curInput[command]) {
                    console.log("Key released:", e.key);
                    e.preventDefault();

                    curInput[command] = false;
                    Game.new_input_this_frame = true;
                }
            }

            document.addEventListener("keydown", keyDownHandler, false);
            document.addEventListener("keyup", keyUpHandler, false);


            Game.update = function() {
                if (this.new_input_this_frame) {
                    jsonInput = JSON.stringify(curInput);
                    this.socket.send(jsonInput);
                    console.log("Sent input: " + jsonInput);
                    this.new_input_this_frame = false;
                }
                Object.entries(new_cells).forEach((index, cell) => {Game.state.discoveredRooms[index] = cell});
            };

            Game.render = function() {
                ctx.clearRect(0, 0, canvas.width, canvas.height);
                Game.renderer.drawBackground();
                this.state.discoveredRooms.forEach((room) => this.renderer.drawRoom(room));
                this.state.discoveredRooms.forEach((room) => this.renderer.drawWalls(room));
                this.renderer.drawPlayer(this.state.player_position, this.state.player_direction);
                this.renderer.drawOtherPlayers();
            }

            Game.run = (function() {
                var ticks_accrued = 0, 
                skipTicks = 1000 / Game.fps,
                maxTickSkips = 10,
                nextGameTick = (new Date).getTime();
            
                return function () {
                    ticks_accrued = 0;
                
                    while ((new Date).getTime() > nextGameTick && ticks_accrued < maxTickSkips) {
                        Game.update();
                        nextGameTick += skipTicks;
                        ticks_accrued++;
                    }
                    if (ticks_accrued != 0){
                        Game.render();
                    }
                };
            })();
        
            // Start the game loop
            Game._intervalId = setInterval(Game.run, 0);
        })
    })
}

function getIndexFromCoords(coords) {
    return coords.y * width + coords.x;
}

async function registerUser(username){
    user_id = username.hashCode();
    const registerUrl = 'http://localhost:8000/register';

    const headers = new Headers({
        'Content-Type': 'application/json'
    })
    const response = await fetch(registerUrl, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify({user_id: user_id})
    });
    return response.json();
}

String.prototype.hashCode = function() {
    var hash = 0, i, chr;
    if (this.length === 0) return hash;
    for (i = 0; i < this.length; i++) {
        chr   = this.charCodeAt(i);
        hash  = ((hash << 5) - hash) + chr;
        hash |= 0; // Convert to 32bit integer
        hash = hash >>> 0;
    }
    return hash;
};

// Rendering

function getCoordsFromIndex(i){
    let y;
    if (i < width) {
        y = 0
        x = i;
    } else {
        y = Math.floor(i/width)
        x = i % (width * y);
    }
    return [x, y];
} 

const canvas = document.getElementById('game')
var ctx = canvas.getContext('2d');
