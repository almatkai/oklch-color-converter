"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.ListRuntime = ListRuntime;
exports.parseListRows = parseListRows;
exports.parseRowIcon = parseRowIcon;
exports.SymbolIcon = SymbolIcon;
const react_1 = __importStar(require("react"));
const primitives_1 = require("../../ui/primitives");
const Markdown_1 = require("../../ui/Markdown");
const detail_runtime_1 = require("./detail-runtime");
const colorConverter_1 = require("../../colorConverter");
function parseListAccessory(value) {
    if (!value || typeof value !== 'object' || !('type' in value))
        return null;
    const node = value;
    if (node.type !== 'List.Dropdown')
        return null;
    const options = [];
    const walk = (entry) => {
        if (entry.type === 'List.Dropdown.Item') {
            const title = textValue(entry.props?.title);
            const value = entry.props?.value !== undefined ? textValue(entry.props.value) : title;
            if (title || value)
                options.push({ title: title || value, value, icon: entry.props?.icon });
            return;
        }
        for (const child of entry.children ?? [])
            walk(child);
    };
    walk(node);
    return {
        actionId: typeof node.props?.actionId === 'string' ? node.props.actionId : undefined,
        options,
    };
}
function cleanSubtitle(value) {
    const subtitle = typeof value === 'string' ? value.trim() : '';
    return /^by\s*$/i.test(subtitle) ? '' : subtitle;
}
function parseRowIcon(value) {
    if (typeof value === 'string' && value.trim()) {
        return { source: value.trim() };
    }
    if (!value || typeof value !== 'object')
        return undefined;
    const icon = value;
    if (typeof icon.fileIcon === 'string' && icon.fileIcon.trim()) {
        return { fileIcon: icon.fileIcon.trim() };
    }
    if ('source' in icon)
        return { source: icon.source, tintColor: icon.tintColor };
    return undefined;
}
function textValue(value) {
    if (typeof value === 'string')
        return value;
    if (typeof value === 'number' || typeof value === 'boolean')
        return String(value);
    if (value && typeof value === 'object') {
        const candidate = value;
        if (candidate.value !== undefined)
            return textValue(candidate.value);
        if (candidate.text !== undefined)
            return textValue(candidate.text);
    }
    return '';
}
function isEmojiGlyph(value) {
    if (typeof value !== 'string')
        return false;
    const text = value.trim();
    if (!text || text.length > 12 || /^Icon\./i.test(text) || /^(?:data:image|https?:|file:)/i.test(text)) {
        return false;
    }
    return /[\p{Extended_Pictographic}\u{1F1E6}-\u{1F1FF}\u2600-\u27BF]/u.test(text);
}
function parseAccessories(value) {
    if (!Array.isArray(value))
        return undefined;
    return value
        .filter((entry) => Boolean(entry) && typeof entry === 'object')
        .map((entry) => ({
        text: entry.text,
        tag: entry.tag,
        date: entry.date,
        icon: entry.icon,
    }));
}
function parseDetailFromChildren(entry) {
    const propDetail = entry.props?.detail;
    if (propDetail && typeof propDetail === 'object' && 'type' in propDetail) {
        return propDetail;
    }
    return (entry.children ?? []).find((child) => child.type === 'List.Item.Detail');
}
function parseListRows(node) {
    const rows = [];
    const walk = (entry, section) => {
        if (entry.type === 'List.Item' || entry.type === 'MenuBarExtra.Item') {
            const title = typeof entry.props?.title === 'string' ? entry.props.title : 'Untitled';
            const subtitle = cleanSubtitle(entry.props?.subtitle);
            const icon = parseRowIcon(entry.props?.icon);
            const id = typeof entry.props?.id === 'string' ? entry.props.id : `${section || 'list'}:${rows.length}`;
            const accessories = parseAccessories(entry.props?.accessories);
            const detail = parseDetailFromChildren(entry);
            const actionIds = Array.isArray(entry.props?.actionIds)
                ? entry.props.actionIds.filter((value) => typeof value === 'string')
                : undefined;
            rows.push({ id, title, subtitle, icon, accessories, detail, section, actionIds });
            return;
        }
        if (entry.type === 'List.Section' || entry.type === 'MenuBarExtra.Section') {
            const nextSection = typeof entry.props?.title === 'string' ? entry.props.title : section;
            for (const child of entry.children ?? []) {
                walk(child, nextSection);
            }
            return;
        }
        for (const child of entry.children ?? []) {
            walk(child, section);
        }
    };
    walk(node);
    return rows;
}
function FileIcon({ path, title }) {
    const [src, setSrc] = (0, react_1.useState)(null);
    (0, react_1.useEffect)(() => {
        let cancelled = false;
        setSrc(null);
        void window.tezbar.getAppIconDataUrl(path)
            .then((value) => {
            if (!cancelled)
                setSrc(value);
        })
            .catch(() => {
            if (!cancelled)
                setSrc(null);
        });
        return () => {
            cancelled = true;
        };
    }, [path]);
    if (src) {
        return react_1.default.createElement("img", { src: src, alt: "", className: "h-7 w-7 shrink-0 rounded-[7px]", draggable: false });
    }
    return (react_1.default.createElement("span", { className: "grid h-7 w-7 shrink-0 place-items-center rounded-[7px] border border-white/10 bg-white/[0.04] text-[11px] font-semibold text-ink-3" }, title.slice(0, 1).toUpperCase()));
}
function resolveIconTintColor(value) {
    if (typeof value === 'string' && value.trim())
        return value.trim();
    if (!value || typeof value !== 'object')
        return undefined;
    const adaptiveColor = value;
    const dark = typeof adaptiveColor.dark === 'string' ? adaptiveColor.dark.trim() : '';
    const light = typeof adaptiveColor.light === 'string' ? adaptiveColor.light.trim() : '';
    return dark || light || undefined;
}
function SymbolIcon({ icon, title, tintColor, }) {
    const token = textValue(icon && typeof icon === 'object' ? icon.source : icon)
        .replace(/^Icon\./, '')
        .toLowerCase();
    const label = token || title.slice(0, 1).toLowerCase();
    const resolvedTintColor = resolveIconTintColor(tintColor);
    if (label.includes('circlefilled')) {
        return (react_1.default.createElement("span", { className: "h-[18px] w-[18px] shrink-0 rounded-full border border-white/25 shadow-[inset_0_1px_0_rgba(255,255,255,0.18),0_1px_3px_rgba(0,0,0,0.22)]", style: { backgroundColor: resolvedTintColor || 'currentColor' }, "aria-hidden": "true" }));
    }
    return (react_1.default.createElement("span", { className: "grid h-5 w-5 shrink-0 place-items-center text-accent-1", style: resolvedTintColor ? { color: resolvedTintColor } : undefined, "aria-hidden": "true" },
        react_1.default.createElement("svg", { viewBox: "0 0 24 24", className: "h-4 w-4", fill: "none", stroke: "currentColor", strokeWidth: "2" }, label.includes('check') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "9" }),
            react_1.default.createElement("path", { d: "m8 12 2.6 2.6L16.5 9" }))) : label.includes('xmark') || label.includes('close') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "9" }),
            react_1.default.createElement("path", { d: "m9 9 6 6m0-6-6 6" }))) : label.includes('minus') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "9" }),
            react_1.default.createElement("path", { d: "M8 12h8" }))) : label.includes('exclamation') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "9" }),
            react_1.default.createElement("path", { d: "M12 7v6m0 4h.01" }))) : label.includes('globe') || label.includes('network') || label.includes('wifi') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "9" }),
            react_1.default.createElement("path", { d: "M3 12h18M12 3c2.5 2.7 3.7 5.7 3.7 9S14.5 18.3 12 21M12 3c-2.5 2.7-3.7 5.7-3.7 9S9.5 18.3 12 21" }))) : label.includes('download') ? (react_1.default.createElement("path", { d: "M12 3v12m0 0 4-4m-4 4-4-4M5 19h14" })) : label.includes('upload') ? (react_1.default.createElement("path", { d: "M12 21V9m0 0 4 4m-4-4-4 4M5 5h14" })) : label.includes('link') ? (react_1.default.createElement("path", { d: "M10 13a5 5 0 0 0 7.1 0l1.4-1.4a5 5 0 0 0-7.1-7.1L10.5 5M14 11a5 5 0 0 0-7.1 0l-1.4 1.4a5 5 0 0 0 7.1 7.1l.9-.9" })) : label.includes('phone') || label.includes('voice') ? (react_1.default.createElement("path", { d: "M22 16.9v3a2 2 0 0 1-2.2 2A19.8 19.8 0 0 1 2.1 4.2 2 2 0 0 1 4.1 2h3a2 2 0 0 1 2 1.7l.5 2.8a2 2 0 0 1-.6 1.8L7.9 9.4a16 16 0 0 0 6.7 6.7l1.1-1.1a2 2 0 0 1 1.8-.6l2.8.5a2 2 0 0 1 1.7 2Z" })) : label.includes('video') || label.includes('play') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("rect", { x: "3", y: "6", width: "18", height: "12", rx: "2" }),
            react_1.default.createElement("path", { d: "m10 9 5 3-5 3V9Z" }))) : label.includes('server') || label.includes('harddrive') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("rect", { x: "3", y: "5", width: "18", height: "14", rx: "2" }),
            react_1.default.createElement("path", { d: "M7 15h.01M11 15h6" }))) : label.includes('question') || label.includes('ping') ? (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "9" }),
            react_1.default.createElement("path", { d: "M9.5 9a2.8 2.8 0 0 1 5.2 1.4c0 2-2.7 2.2-2.7 4M12 18h.01" }))) : (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("circle", { cx: "12", cy: "12", r: "8" }),
            react_1.default.createElement("path", { d: "M12 8v8M8 12h8" }))))));
}
function GlyphIcon({ value }) {
    return (react_1.default.createElement("span", { className: "grid h-8 w-8 shrink-0 place-items-center rounded-[8px] border border-white/[0.08] bg-white/[0.045] text-[21px] leading-none shadow-[inset_0_1px_0_rgba(255,255,255,0.08)]", "aria-hidden": "true" }, value));
}
function RowIcon({ row }) {
    if (row.swatch) {
        return (react_1.default.createElement("span", { className: "h-[18px] w-[18px] shrink-0 rounded-full border border-white/30 shadow-[0_0_14px_rgba(255,255,255,0.18)]", style: { background: row.swatch }, "aria-hidden": "true" }));
    }
    if (row.icon?.fileIcon)
        return react_1.default.createElement(FileIcon, { path: row.icon.fileIcon, title: row.title });
    if (isEmojiGlyph(row.icon?.source))
        return react_1.default.createElement(GlyphIcon, { value: row.icon.source });
    if (typeof row.icon?.source === 'string' &&
        /^(?:data:image|https?:|file:)/i.test(row.icon.source)) {
        return react_1.default.createElement("img", { src: row.icon.source, alt: "", className: "h-5 w-5 shrink-0", draggable: false });
    }
    if (row.icon?.source) {
        return react_1.default.createElement(SymbolIcon, { icon: row.icon.source, title: row.title, tintColor: row.icon.tintColor });
    }
    return null;
}
function accessoryText(accessory) {
    const tag = textValue(accessory.tag);
    if (tag)
        return tag;
    const text = textValue(accessory.text);
    if (text)
        return text;
    if (accessory.date) {
        const date = new Date(String(accessory.date));
        if (!Number.isNaN(date.getTime()))
            return date.toLocaleDateString();
    }
    return '';
}
function Accessory({ accessory, }) {
    const value = accessoryText(accessory);
    if (!value)
        return null;
    const isTag = Boolean(accessory.tag);
    const tagColor = textValue(accessory.tag && typeof accessory.tag === 'object'
        ? accessory.tag.color
        : '').toLowerCase();
    const colorClass = tagColor.includes('blue')
        ? 'bg-blue-500/20 text-blue-300'
        : tagColor.includes('orange')
            ? 'bg-orange-500/20 text-orange-300'
            : tagColor.includes('red')
                ? 'bg-red-500/20 text-red-300'
                : tagColor.includes('green')
                    ? 'bg-emerald-500/20 text-emerald-300'
                    : 'bg-white/10 text-ink-2';
    return (react_1.default.createElement("span", { className: isTag
            ? `max-w-[190px] shrink-0 truncate rounded-[8px] px-2 py-1 text-[11px] font-medium ${colorClass}`
            : 'max-w-[240px] shrink-0 truncate text-[12px] text-ink-3' }, value));
}
function markdownFromListDetail(detail) {
    if (!detail)
        return '';
    if (typeof detail.props?.markdown === 'string')
        return detail.props.markdown;
    for (const child of detail.children ?? []) {
        if (typeof child.props?.markdown === 'string')
            return child.props.markdown;
    }
    return '';
}
function metadataFromListDetail(detail) {
    if (!detail)
        return undefined;
    const propMetadata = detail.props?.metadata;
    if (propMetadata && typeof propMetadata === 'object' && 'type' in propMetadata) {
        return propMetadata;
    }
    if (detail.metadata)
        return detail.metadata;
    return (detail.children ?? []).find((child) => child.type === 'List.Item.Detail.Metadata');
}
function InlineMetadata({ root }) {
    return (react_1.default.createElement("div", { className: "h-full overflow-y-auto px-4 py-4" },
        react_1.default.createElement("div", { className: "max-w-[520px]" }, (root.children ?? []).map((child, index) => (react_1.default.createElement(detail_runtime_1.MetadataItem, { key: index, node: child }))))));
}
function ListDetailPane({ row }) {
    const markdown = markdownFromListDetail(row?.detail);
    const metadata = metadataFromListDetail(row?.detail);
    if (!row?.detail) {
        return (react_1.default.createElement("div", { className: "flex h-full items-center justify-center px-6 text-center text-[12px] text-ink-4" }, "Select an item to see details."));
    }
    return (react_1.default.createElement("div", { className: "flex h-full min-h-0 overflow-hidden" },
        metadata && !markdown ? (react_1.default.createElement(InlineMetadata, { root: metadata })) : (react_1.default.createElement("div", { className: "min-h-0 flex-1 overflow-y-auto px-4 py-4" }, markdown ? (react_1.default.createElement("article", { className: "prose prose-invert max-w-none text-[13px] leading-relaxed" },
            react_1.default.createElement(Markdown_1.Markdown, { text: markdown, className: "text-[13px] leading-relaxed" }))) : metadata ? null : (react_1.default.createElement("div", { className: "text-[12px] text-ink-4" }, "No detail content")))),
        metadata && markdown ? react_1.default.createElement(detail_runtime_1.MetadataSidebar, { root: metadata }) : null));
}
function groupBySection(rows) {
    const groups = [];
    let currentSection = null;
    for (const row of rows) {
        if (row.section !== currentSection || groups.length === 0) {
            currentSection = row.section;
            groups.push({ section: row.section, items: [] });
        }
        const last = groups[groups.length - 1];
        if (last)
            last.items.push(row);
    }
    return groups;
}
function clientSideFilter(rows, query) {
    const q = query.trim().toLowerCase();
    if (!q)
        return rows;
    return rows.filter((row) => {
        return [row.title, row.subtitle, row.section]
            .filter((value) => Boolean(value))
            .some((value) => value.toLowerCase().includes(q));
    });
}
function isColorConversionCommand(commandName, title, navigationTitle) {
    const haystack = `${commandName} ${title} ${navigationTitle}`.toLowerCase();
    return haystack.includes('convert') && haystack.includes('color');
}
async function copyRuntimeText(value) {
    if (!value)
        return;
    await window.tezbar.clipboardWriteText(value);
}
function ListRuntime({ root, title, commandName, onBack, onRunPrimaryAction, actions, onOpenActions, onSearchTextChanged, onLoadMore, }) {
    const inputRef = (0, react_1.useRef)(null);
    const listRef = (0, react_1.useRef)(null);
    const rows = (0, react_1.useMemo)(() => {
        const parsed = parseListRows(root);
        return parsed;
    }, [root]);
    const searchBarPlaceholder = (0, react_1.useMemo)(() => {
        return typeof root.props?.searchBarPlaceholder === 'string' && root.props.searchBarPlaceholder.trim().length > 0
            ? root.props.searchBarPlaceholder
            : 'Search applications';
    }, [root.props]);
    const navigationTitle = typeof root.props?.navigationTitle === 'string' && root.props.navigationTitle.trim()
        ? root.props.navigationTitle
        : title;
    const searchAccessory = (0, react_1.useMemo)(() => parseListAccessory(root.props?.searchBarAccessory), [root.props?.searchBarAccessory]);
    const hasServerSearch = onSearchTextChanged !== undefined && root.props?.__hasServerSearch === true;
    const hasMore = root.props?.__hasMore === true;
    const isColorConversionSurface = isColorConversionCommand(commandName, title, navigationTitle);
    const hasManySearchOptions = (searchAccessory?.options.length ?? 0) > 4;
    const hasSearchPlaceholder = typeof root.props?.searchBarPlaceholder === 'string' &&
        root.props.searchBarPlaceholder.trim().length > 0;
    const hasSearchText = typeof root.props?.searchText === 'string';
    const hasClientSearchableRows = rows.length > 0;
    const shouldShowSearch = isColorConversionSurface ||
        hasServerSearch ||
        Boolean(searchAccessory) ||
        hasSearchPlaceholder ||
        hasSearchText ||
        hasClientSearchableRows ||
        !root.props?.navigationTitle;
    const [query, setQuery] = (0, react_1.useState)(typeof root.props?.searchText === 'string' ? root.props.searchText : '');
    const [selected, setSelected] = (0, react_1.useState)(0);
    const [accessoryValue, setAccessoryValue] = (0, react_1.useState)(searchAccessory?.options[0]?.value ?? '');
    const [loadingMore, setLoadingMore] = (0, react_1.useState)(false);
    const debounceTimer = (0, react_1.useRef)(null);
    const lastSentQuery = (0, react_1.useRef)('');
    const loadingMoreRef = (0, react_1.useRef)(false);
    (0, react_1.useEffect)(() => {
        inputRef.current?.focus();
    }, []);
    (0, react_1.useEffect)(() => {
        if (typeof root.props?.searchText === 'string') {
            setQuery(root.props.searchText);
        }
    }, [root.props?.searchText]);
    (0, react_1.useEffect)(() => {
        setAccessoryValue((current) => {
            if (!searchAccessory || searchAccessory.options.length === 0)
                return '';
            return searchAccessory.options.some((option) => option.value === current)
                ? current
                : searchAccessory.options[0]?.value ?? '';
        });
    }, [searchAccessory]);
    (0, react_1.useEffect)(() => {
        if (query === lastSentQuery.current)
            return;
        if (debounceTimer.current) {
            clearTimeout(debounceTimer.current);
        }
        debounceTimer.current = setTimeout(() => {
            lastSentQuery.current = query;
            void onSearchTextChanged(query);
        }, 200);
        return () => {
            if (debounceTimer.current) {
                clearTimeout(debounceTimer.current);
            }
        };
    }, [query, onSearchTextChanged]);
    const colorConversionRows = (0, react_1.useMemo)(() => {
        if (!isColorConversionSurface)
            return [];
        return (0, colorConverter_1.buildColorConversionResults)(query).map((result) => {
            const copyText = result.action.type === 'copy-text' ? result.action.text : result.title;
            return {
                id: `runtime:${result.id}`,
                title: result.title,
                subtitle: result.subtitle,
                copyText,
                swatch: copyText,
            };
        });
    }, [isColorConversionSurface, query]);
    const filteredRows = (0, react_1.useMemo)(() => {
        if (colorConversionRows.length > 0)
            return colorConversionRows;
        if (hasServerSearch)
            return rows;
        return clientSideFilter(rows, query);
    }, [colorConversionRows, hasServerSearch, rows, query]);
    const groupedSections = (0, react_1.useMemo)(() => groupBySection(filteredRows), [filteredRows]);
    const selectedRow = filteredRows[selected];
    const selectedAction = actions.find((action) => selectedRow?.actionIds?.[0] === action.id);
    const selectedActionLabel = selectedRow?.copyText ? 'Copy' : selectedAction?.title || 'Run';
    const hasDetails = filteredRows.some((row) => row.detail);
    const usesEmojiGlyphs = filteredRows.some((row) => isEmojiGlyph(row.icon?.source));
    const requestMore = async () => {
        if (!hasMore || loadingMoreRef.current)
            return;
        loadingMoreRef.current = true;
        setLoadingMore(true);
        try {
            await onLoadMore();
        }
        finally {
            loadingMoreRef.current = false;
            setLoadingMore(false);
        }
    };
    const onListScroll = (event) => {
        const { clientHeight, scrollHeight, scrollTop } = event.currentTarget;
        const remaining = scrollHeight - scrollTop - clientHeight;
        if (remaining <= Math.max(120, clientHeight * 0.25))
            void requestMore();
    };
    (0, react_1.useEffect)(() => {
        if (selected >= filteredRows.length) {
            setSelected(Math.max(0, filteredRows.length - 1));
        }
    }, [filteredRows.length, selected]);
    (0, react_1.useEffect)(() => {
        const el = listRef.current?.querySelector(`[data-idx="${selected}"]`);
        if (el) {
            el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        }
    }, [selected]);
    const onKeyDown = (e) => {
        const isActionsShortcut = e.metaKey &&
            !e.ctrlKey &&
            !e.altKey &&
            !e.shiftKey &&
            e.key.toLowerCase() === 'k';
        if (isActionsShortcut) {
            e.preventDefault();
            e.stopPropagation();
            onOpenActions(filteredRows[selected]?.actionIds);
            return;
        }
        if (filteredRows.length === 0)
            return;
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            const next = Math.min(selected + 1, filteredRows.length - 1);
            setSelected(next);
            if (hasMore && next >= filteredRows.length - 3)
                void requestMore();
            return;
        }
        if (e.key === 'ArrowUp') {
            e.preventDefault();
            setSelected((i) => Math.max(i - 1, 0));
            return;
        }
        if (e.key === 'Enter' && !e.repeat) {
            e.preventDefault();
            const row = filteredRows[selected];
            if (row?.copyText) {
                void copyRuntimeText(row.copyText);
                return;
            }
            const actionId = row?.actionIds?.[0];
            if (actionId)
                onRunPrimaryAction(actionId);
            return;
        }
    };
    const emptyView = (0, react_1.useMemo)(() => {
        const candidate = (root.children ?? []).find((entry) => entry.type === 'List.EmptyView');
        if (!candidate)
            return null;
        return {
            title: typeof candidate.props?.title === 'string' ? candidate.props.title : 'No results',
            description: typeof candidate.props?.description === 'string' ? candidate.props.description : '',
        };
    }, [root.children]);
    const hasQuery = query.trim().length > 0;
    const isFirstTimePackageLoad = filteredRows.length === 0 &&
        emptyView?.title === 'Loading Packages';
    const emptyTitle = isColorConversionSurface
        ? hasQuery
            ? 'No color formats'
            : 'Enter a color'
        : hasQuery
            ? 'No matching applications'
            : emptyView?.title || 'No list items';
    const emptyDescription = isColorConversionSurface
        ? hasQuery
            ? 'Try HEX, RGB, RGBA, HSL, or HSLA.'
            : 'Type a color value to convert it.'
        : hasQuery
            ? 'Try a different app name or keyword.'
            : emptyView?.description || '';
    return (react_1.default.createElement("div", { className: "flex h-full min-h-0 w-full flex-col gap-2 outline-none animate-tezbar-scale-in", onKeyDown: onKeyDown },
        react_1.default.createElement("div", { className: "glass-card shrink-0 px-4 py-3 animate-tezbar-scale-in" },
            react_1.default.createElement(primitives_1.ViewHeader, { title: navigationTitle, onBack: onBack }),
            shouldShowSearch ? (react_1.default.createElement("div", { className: (0, primitives_1.cx)('mt-2 flex gap-2', hasManySearchOptions ? 'flex-col items-stretch' : 'items-center') },
                react_1.default.createElement("div", { className: "min-w-0 flex-1" },
                    react_1.default.createElement("input", { ref: inputRef, type: "text", value: query, onChange: (event) => {
                            setQuery(event.target.value);
                            setSelected(0);
                        }, placeholder: searchBarPlaceholder, className: "h-8 w-full rounded-tezbar-chip border border-white/10 bg-white/[0.04] px-2.5 text-[12px] text-ink-1 placeholder:text-ink-4 outline-none transition focus:border-white/20 focus:bg-white/[0.06]", "aria-label": searchBarPlaceholder })),
                searchAccessory && searchAccessory.options.length > 0 ? (react_1.default.createElement("div", { className: (0, primitives_1.cx)('flex min-w-0 max-w-full items-center gap-1 overflow-x-auto rounded-tezbar-field p-0.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden', hasManySearchOptions ? 'w-full' : 'shrink-0') }, searchAccessory.options.map((option) => {
                    const active = accessoryValue === option.value;
                    return (react_1.default.createElement("button", { key: option.value, type: "button", onClick: () => {
                            setAccessoryValue(option.value);
                            setSelected(0);
                            if (searchAccessory.actionId) {
                                onRunPrimaryAction(searchAccessory.actionId, { value: option.value });
                            }
                        }, className: active
                            ? 'shrink-0 rounded-tezbar-chip bg-white/[0.14] px-2.5 py-1.5 text-[11px] font-medium text-ink-1 transition'
                            : 'shrink-0 rounded-tezbar-chip px-2.5 py-1.5 text-[11px] text-ink-3 transition hover:bg-white/[0.06] hover:text-ink-1' },
                        isEmojiGlyph(option.icon) ? (react_1.default.createElement("span", { className: "mr-1.5 inline-grid h-4 w-4 place-items-center text-[13px] leading-none", "aria-hidden": "true" }, option.icon)) : null,
                        option.title));
                }))) : null)) : null),
        react_1.default.createElement("div", { ref: listRef, className: `glass-card min-h-0 flex-1 overflow-hidden animate-tezbar-scale-in ${hasDetails ? 'grid grid-cols-[minmax(280px,40%)_minmax(0,1fr)] px-2 py-2' : 'flex flex-col px-4 py-3'}` }, filteredRows.length === 0 ? (react_1.default.createElement("div", { className: "flex h-full items-center justify-center text-center" },
            react_1.default.createElement("div", null,
                react_1.default.createElement("p", { className: "text-[13px] text-ink-2" }, emptyTitle),
                emptyDescription ? (react_1.default.createElement("p", { className: "mt-1 text-[11px] text-ink-4" }, emptyDescription)) : null,
                isFirstTimePackageLoad ? (react_1.default.createElement("p", { className: "mx-auto mt-3 max-w-sm text-[11px] leading-5 text-ink-3" }, "First-time setup may take a few minutes while the package catalog is downloaded and indexed. Future searches will open much faster from the local cache.")) : null))) : (react_1.default.createElement(react_1.default.Fragment, null,
            react_1.default.createElement("div", { onScroll: onListScroll, className: hasDetails ? 'min-h-0 overflow-y-auto pr-2' : 'min-h-0 flex-1 overflow-y-auto pr-0.5' }, groupedSections.map((group, groupIdx) => (react_1.default.createElement("div", { key: groupIdx, className: "mb-1" },
                group.section ? (react_1.default.createElement("div", { className: "px-3 pb-1.5 pt-1 text-[11px] font-semibold text-ink-4 select-none" }, group.section)) : null,
                react_1.default.createElement("ul", { className: "flex flex-col gap-0.5" }, group.items.map((row, localIdx) => {
                    const globalIdx = groupedSections
                        .slice(0, groupIdx)
                        .reduce((sum, g) => sum + g.items.length, 0) + localIdx;
                    return (react_1.default.createElement("li", { key: row.id },
                        react_1.default.createElement("button", { type: "button", "data-idx": globalIdx, onMouseEnter: () => setSelected(globalIdx), onClick: () => {
                                setSelected(globalIdx);
                                if (row.copyText) {
                                    void copyRuntimeText(row.copyText);
                                }
                                else if (row.actionIds?.[0]) {
                                    onRunPrimaryAction(row.actionIds[0]);
                                }
                            }, className: (0, primitives_1.cx)('w-full rounded-tezbar-row px-3 text-left transition', usesEmojiGlyphs ? 'py-2' : 'py-2.5', globalIdx === selected
                                ? 'bg-white/[0.16] text-ink-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
                                : 'text-ink-2 hover:bg-white/[0.06]') },
                            react_1.default.createElement("span", { className: (0, primitives_1.cx)('flex min-w-0 items-center', usesEmojiGlyphs ? 'gap-3' : 'gap-2.5') },
                                react_1.default.createElement(RowIcon, { row: row }),
                                react_1.default.createElement("span", { className: "min-w-0 flex-1" },
                                    react_1.default.createElement("p", { className: "truncate text-[13px] font-medium" }, row.title),
                                    row.subtitle ? react_1.default.createElement("p", { className: "truncate text-[11px] text-ink-3" }, row.subtitle) : null),
                                row.accessories?.map((accessory, index) => {
                                    return react_1.default.createElement(Accessory, { key: index, accessory: accessory });
                                })))));
                })))))),
            hasDetails ? (react_1.default.createElement("div", { className: "min-h-0 overflow-hidden rounded-tezbar-row border border-white/[0.06] bg-white/[0.02]" },
                react_1.default.createElement(ListDetailPane, { row: selectedRow }))) : null))),
        react_1.default.createElement("div", { className: "glass-card flex shrink-0 items-center justify-between gap-3 px-4 py-2 animate-tezbar-scale-in" },
            react_1.default.createElement("span", { className: "flex items-center gap-2 text-[11px] font-medium text-ink-4" },
                filteredRows.length,
                " item",
                filteredRows.length === 1 ? '' : 's',
                hasMore ? (react_1.default.createElement("button", { type: "button", disabled: loadingMore, onClick: () => void requestMore(), className: "ml-1 text-[10.5px] font-semibold normal-case tracking-normal text-ink-2 transition hover:text-ink-1 disabled:opacity-50" }, loadingMore ? 'Loading...' : 'Load more')) : null),
            react_1.default.createElement(primitives_1.HintBar, null,
                react_1.default.createElement(primitives_1.Hint, { label: "Select", keys: react_1.default.createElement(react_1.default.Fragment, null,
                        react_1.default.createElement(primitives_1.Kbd, null, "\u2191"),
                        react_1.default.createElement(primitives_1.Kbd, null, "\u2193")) }),
                react_1.default.createElement(primitives_1.Hint, { label: selectedActionLabel, keys: react_1.default.createElement(primitives_1.Kbd, null, "\u21B5") }),
                react_1.default.createElement(primitives_1.Hint, { label: "Actions", keys: react_1.default.createElement(react_1.default.Fragment, null,
                        react_1.default.createElement(primitives_1.Kbd, null, "\u2318"),
                        react_1.default.createElement(primitives_1.Kbd, null, "K")) }),
                react_1.default.createElement(primitives_1.Hint, { label: "Back", keys: react_1.default.createElement(primitives_1.Kbd, null, "Esc") })))));
}
