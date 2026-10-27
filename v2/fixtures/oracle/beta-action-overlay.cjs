"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ActionPanelOverlay = ActionPanelOverlay;
const react_1 = require("react");
function ActionPanelOverlay({ actions, onClose, onExecute, }) {
    const [selected, setSelected] = (0, react_1.useState)(0);
    const [filter, setFilter] = (0, react_1.useState)('');
    const visibleActions = (0, react_1.useMemo)(() => {
        const q = filter.trim().toLowerCase();
        if (!q)
            return actions;
        return actions.filter((action) => action.title.toLowerCase().includes(q));
    }, [actions, filter]);
    (0, react_1.useEffect)(() => {
        setSelected(0);
    }, [filter]);
    (0, react_1.useEffect)(() => {
        const onKey = (event) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                onClose();
                return;
            }
            if (event.key === 'ArrowDown') {
                event.preventDefault();
                setSelected((index) => Math.min(index + 1, Math.max(visibleActions.length - 1, 0)));
            }
            if (event.key === 'ArrowUp') {
                event.preventDefault();
                setSelected((index) => Math.max(index - 1, 0));
            }
            if (event.key === 'Enter' && visibleActions[selected]) {
                event.preventDefault();
                onExecute(visibleActions[selected]);
            }
        };
        window.addEventListener('keydown', onKey, true);
        return () => window.removeEventListener('keydown', onKey, true);
    }, [onClose, onExecute, selected, visibleActions]);
    return (React.createElement("div", { className: "fixed inset-0 z-50 bg-black/50", onClick: onClose },
        React.createElement("div", { className: "absolute bottom-12 right-3 w-80 max-h-[65vh] overflow-hidden rounded-tezbar-card border border-white/10 bg-black/75 backdrop-blur-xl", onClick: (event) => event.stopPropagation() },
            React.createElement("div", { className: "border-b border-white/10 px-3 py-2" },
                React.createElement("input", { value: filter, onChange: (event) => setFilter(event.target.value), placeholder: "Search actions", className: "glass-field text-[12px]", autoFocus: true })),
            React.createElement("ul", { className: "max-h-[48vh] overflow-y-auto p-1" },
                visibleActions.map((action, index) => (React.createElement("li", { key: action.id },
                    React.createElement("button", { type: "button", onClick: () => onExecute(action), onMouseEnter: () => setSelected(index), className: `w-full rounded-tezbar-row px-3 py-2 text-left text-[12px] transition ${index === selected ? 'bg-white/15 text-ink-1' : 'text-ink-2 hover:bg-white/8'}` }, action.title)))),
                visibleActions.length === 0 ? (React.createElement("li", { className: "px-3 py-3 text-[12px] text-ink-4" }, "No matching actions")) : null))));
}
