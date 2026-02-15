import init, { Game, build_timestamp } from './pkg/chess.js?v=4';

let game = null;

async function initialize() {
  await init();
  postMessage({ type: 'init', timestamp: build_timestamp() });
}

function handle(msg) {
  const { id, method, args } = msg;
  let result;
  switch (method) {
    case 'new_game':
      game = new Game();
      result = null;
      break;
    case 'get_board_state':
      result = game.get_board_state();
      break;
    case 'make_move':
      result = game.make_move(...args);
      break;
    case 'make_ai_move':
      result = game.make_ai_move();
      break;
    case 'get_hint':
      result = game.get_hint(...args);
      break;
    case 'get_legal_moves_for_square':
      result = game.get_legal_moves_for_square(...args);
      break;
    case 'get_eval_breakdown':
      result = game.get_eval_breakdown();
      break;
    case 'get_last_evals':
      result = Number(game.get_last_evals());
      break;
    case 'set_module':
      game.set_module(...args);
      result = null;
      break;
    case 'set_depth':
      game.set_depth(...args);
      result = null;
      break;
    case 'set_auto_deepen':
      game.set_auto_deepen(args[0], BigInt(args[1]));
      result = null;
      break;
    default:
      postMessage({ id, error: 'unknown method: ' + method });
      return;
  }
  postMessage({ id, result });
}

onmessage = (e) => handle(e.data);

initialize().catch(err => {
  postMessage({ type: 'init_error', error: err.message });
});
