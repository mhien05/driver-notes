<script lang="ts">
  import { onMount, tick } from "svelte";
  import Icon from "./lib/Icon.svelte";
  import {
    api,
    ApiError,
    type AuthResponse,
    type Entry,
    type EntryInput,
    type EntryKind,
    type ExpenseCategory,
    type Shift,
  } from "./lib/api";

  type Tab = "home" | "history" | "shifts" | "account";
  type Snapshot = { entries: Entry[]; shifts: Shift[]; savedAt: number };
  const nav: { id: Tab; label: string; icon: string }[] = [
    { id: "home", label: "Tổng quan", icon: "home" },
    { id: "history", label: "Giao dịch", icon: "list" },
    { id: "shifts", label: "Ca làm", icon: "clock" },
    { id: "account", label: "Tài khoản", icon: "user" },
  ];
  const money = new Intl.NumberFormat("vi-VN");
  const vnd = (n: number) => `${money.format(n)} ₫`;
  const dayKey = (n: number) => {
    const d = new Date(n);
    return `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
  };
  const dayLabel = (n: number) =>
    new Intl.DateTimeFormat("vi-VN", {
      weekday: "long",
      day: "numeric",
      month: "long",
    }).format(n);
  const dateTime = (n: number) =>
    new Intl.DateTimeFormat("vi-VN", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(n);
  const clock = (n: number) =>
    new Intl.DateTimeFormat("vi-VN", {
      hour: "2-digit",
      minute: "2-digit",
    }).format(n);
  const duration = (a: number, b: number) => {
    const m = Math.max(0, Math.floor((b - a) / 60000));
    return `${Math.floor(m / 60)} giờ ${m % 60} phút`;
  };
  const entryTitle = (e: Entry) =>
    e.kind === "delivery"
      ? `Giao hàng · ${e.deliveryCount} đơn`
      : e.kind === "bonus"
        ? "Tiền thưởng"
        : { fuel: "Xăng xe", food: "Ăn uống", other: "Chi phí khác" }[
            e.expenseCategory ?? "other"
          ];
  const entryIcon = (e: Entry) =>
    e.kind === "delivery"
      ? "truck"
      : e.kind === "bonus"
        ? "gift"
        : e.expenseCategory === "fuel"
          ? "fuel"
          : "wallet";

  let session: AuthResponse | null = null;
  let entries: Entry[] = [];
  let shifts: Shift[] = [];
  let snapshotAt: number | null = null;
  let activeTab: Tab = "home";
  let loading = true;
  let busy = false;
  let offline = false;
  let error = "";
  let notice = "";
  let now = Date.now();
  let authMode: "login" | "register" = "login";
  let email = "";
  let password = "";
  let authError = "";
  let showForm = false;
  let editingId: string | null = null;
  let kind: EntryKind = "delivery";
  let amount = "";
  let tip = "";
  let deliveryCount = 1;
  let expenseCategory: ExpenseCategory = "fuel";
  let note = "";
  let occurredAt = "";
  let selectedShiftId = "";
  let formError = "";
  let historyFilter: "all" | EntryKind = "all";

  $: activeShift = shifts.find((s) => s.endedAt === null) ?? null;
  $: todayEntries = entries.filter((e) => dayKey(e.occurredAt) === dayKey(now));
  $: todayIncome = todayEntries.reduce(
    (sum, e) => sum + (e.kind === "expense" ? 0 : e.amountVnd + e.tipVnd),
    0,
  );
  $: todayExpense = todayEntries.reduce(
    (sum, e) => sum + (e.kind === "expense" ? e.amountVnd : 0),
    0,
  );
  $: todayOrders = todayEntries.reduce(
    (sum, e) => sum + (e.kind === "delivery" ? (e.deliveryCount ?? 0) : 0),
    0,
  );
  $: week = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(now);
    d.setHours(0, 0, 0, 0);
    d.setDate(d.getDate() - (6 - i));
    return {
      label: new Intl.DateTimeFormat("vi-VN", { weekday: "short" }).format(d),
      value: entries
        .filter((e) => dayKey(e.occurredAt) === dayKey(d.getTime()))
        .reduce(
          (sum, e) =>
            sum +
            (e.kind === "expense" ? -e.amountVnd : e.amountVnd + e.tipVnd),
          0,
        ),
      current: i === 6,
    };
  });
  $: weekMax = Math.max(1, ...week.map((d) => Math.max(0, d.value)));
  $: filteredEntries = entries.filter(
    (e) => historyFilter === "all" || e.kind === historyFilter,
  );

  const snapshotKey = (id: string) => `driver-notes:snapshot:${id}`;
  const sessionKey = "driver-notes:session";
  function readStoredSession() {
    try {
      return localStorage.getItem(sessionKey) ?? sessionStorage.getItem(sessionKey);
    } catch {
      try { return sessionStorage.getItem(sessionKey); } catch { return null; }
    }
  }
  function saveStoredSession(value: AuthResponse) {
    try {
      localStorage.setItem(sessionKey, JSON.stringify(value));
      sessionStorage.removeItem(sessionKey);
    } catch {
      try { sessionStorage.setItem(sessionKey, JSON.stringify(value)); } catch { /* Giữ phiên trong bộ nhớ hiện tại. */ }
    }
  }
  function clearStoredSession() {
    try { localStorage.removeItem(sessionKey); } catch { /* Bộ nhớ có thể bị chặn. */ }
    try { sessionStorage.removeItem(sessionKey); } catch { /* Bộ nhớ có thể bị chặn. */ }
  }
  function readSnapshot(id: string) {
    try {
      const raw = localStorage.getItem(snapshotKey(id));
      if (!raw) return false;
      const saved = JSON.parse(raw) as Snapshot;
      entries = saved.entries;
      shifts = saved.shifts;
      snapshotAt = saved.savedAt;
      return true;
    } catch {
      return false;
    }
  }
  async function loadData() {
    if (!session) return;
    const current = session;
    try {
      const [nextEntries, nextShifts] = await Promise.all([
        api<Entry[]>("/entries", current.token),
        api<Shift[]>("/shifts", current.token),
      ]);
      entries = nextEntries;
      shifts = nextShifts;
      snapshotAt = Date.now();
      offline = false;
      error = "";
      try {
        localStorage.setItem(
          snapshotKey(current.userId),
          JSON.stringify({
            entries,
            shifts,
            savedAt: snapshotAt,
          } satisfies Snapshot),
        );
      } catch { /* Dữ liệu online vẫn sử dụng được khi bộ nhớ thiết bị đầy. */ }
    } catch (caught) {
      if (caught instanceof ApiError && caught.status === 401) {
        session = null;
        clearStoredSession();
        error = "Phiên đăng nhập đã hết hạn. Vui lòng đăng nhập lại.";
      } else if (caught instanceof ApiError && caught.status === 0) {
        offline = true;
        if (!readSnapshot(current.userId))
          error = "Chưa có dữ liệu đã lưu để xem khi mất mạng.";
      } else
        error =
          caught instanceof Error ? caught.message : "Không thể tải dữ liệu.";
    } finally {
      loading = false;
    }
  }
  onMount(() => {
    const saved = readStoredSession();
    if (saved) {
      try {
        const parsed = JSON.parse(saved) as AuthResponse;
        if (parsed.expiresAt > Date.now()) {
          saveStoredSession(parsed);
          session = parsed;
          readSnapshot(parsed.userId);
          void loadData();
        } else {
          clearStoredSession();
          loading = false;
        }
      } catch {
        loading = false;
      }
    } else loading = false;
    const timer = window.setInterval(() => {
      now = Date.now();
    }, 60_000);
    const reconnect = () => {
      if (session) void loadData();
    };
    window.addEventListener("online", reconnect);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("online", reconnect);
    };
  });
  async function authenticate() {
    authError = "";
    if (authMode === "register" && password.length < 12) {
      authError = "Mật khẩu cần ít nhất 12 ký tự.";
      return;
    }
    busy = true;
    try {
      const next = await api<AuthResponse>(`/auth/${authMode}`, undefined, {
        method: "POST",
        body: JSON.stringify({ email: email.trim(), password }),
      });
      session = next;
      saveStoredSession(next);
      password = "";
      loading = true;
      await loadData();
    } catch (caught) {
      authError =
        caught instanceof ApiError && caught.status === 401
          ? "Email hoặc mật khẩu không đúng."
          : caught instanceof ApiError && caught.status === 409
            ? "Email này đã được đăng ký."
            : caught instanceof Error
              ? caught.message
              : "Không thể đăng nhập.";
    } finally {
      busy = false;
    }
  }
  async function logout() {
    if (!session) return;
    busy = true;
    try {
      if (!offline)
        await api<void>("/auth/logout", session.token, { method: "POST" });
    } catch {
      /* Vẫn xóa phiên trên thiết bị. */
    }
    clearStoredSession();
    session = null;
    entries = [];
    shifts = [];
    snapshotAt = null;
    activeTab = "home";
    busy = false;
  }
  async function changeShift() {
    if (!session || busy || offline) return;
    if (activeShift && !window.confirm("Kết thúc ca làm hiện tại?")) return;
    const ending = !!activeShift;
    busy = true;
    error = "";
    try {
      await api<Shift>(
        activeShift ? `/shifts/${activeShift.id}/end` : "/shifts",
        session.token,
        { method: "POST" },
      );
      await loadData();
      notice = ending ? "Đã kết thúc ca làm." : "Ca làm đã bắt đầu.";
    } catch (caught) {
      error =
        caught instanceof Error ? caught.message : "Không thể cập nhật ca làm.";
    } finally {
      busy = false;
    }
  }
  function localInputDate(n: number) {
    const d = new Date(n),
      pad = (v: number) => String(v).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }
  async function openForm(entry?: Entry, initialKind: EntryKind = "delivery") {
    editingId = entry?.id ?? null;
    kind = entry?.kind ?? initialKind;
    amount = entry ? String(entry.amountVnd) : "";
    tip = entry?.tipVnd ? String(entry.tipVnd) : "";
    deliveryCount = entry?.deliveryCount ?? 1;
    expenseCategory = entry?.expenseCategory ?? "fuel";
    note = entry?.note ?? "";
    occurredAt = localInputDate(entry?.occurredAt ?? Date.now());
    selectedShiftId = entry?.shiftId ?? activeShift?.id ?? "";
    formError = "";
    showForm = true;
    await tick();
    document.getElementById("entry-amount")?.focus();
  }
  async function saveEntry() {
    if (!session || busy || offline) return;
    const value = Number(amount),
      tipValue = Number(tip || 0),
      occurred = new Date(occurredAt).getTime();
    if (
      !Number.isSafeInteger(value) ||
      value <= 0 ||
      value > 1_000_000_000_000
    ) {
      formError = "Nhập số tiền hợp lệ lớn hơn 0.";
      return;
    }
    if (
      kind === "delivery" &&
      (!Number.isSafeInteger(tipValue) ||
        tipValue < 0 ||
        tipValue > 1_000_000_000_000)
    ) {
      formError = "Tiền tip không hợp lệ.";
      return;
    }
    if (!Number.isFinite(occurred)) {
      formError = "Chọn thời gian phát sinh hợp lệ.";
      return;
    }
    if ([...note].length > 500) {
      formError = "Ghi chú tối đa 500 ký tự.";
      return;
    }
    const input: EntryInput = {
      shiftId: selectedShiftId || null,
      kind,
      deliveryCount: kind === "delivery" ? Number(deliveryCount) : null,
      expenseCategory: kind === "expense" ? expenseCategory : null,
      amountVnd: value,
      tipVnd: kind === "delivery" ? tipValue : 0,
      occurredAt: occurred,
      note: note.trim(),
    };
    busy = true;
    formError = "";
    try {
      await api<Entry>(
        editingId ? `/entries/${editingId}` : "/entries",
        session.token,
        { method: editingId ? "PUT" : "POST", body: JSON.stringify(input) },
      );
      showForm = false;
      await loadData();
      notice = editingId ? "Đã cập nhật giao dịch." : "Đã lưu giao dịch.";
    } catch (caught) {
      formError =
        caught instanceof Error ? caught.message : "Không thể lưu giao dịch.";
    } finally {
      busy = false;
    }
  }
  async function deleteEntry(entry: Entry) {
    if (
      !session ||
      busy ||
      offline ||
      !window.confirm(`Xóa giao dịch “${entryTitle(entry)}” này?`)
    )
      return;
    busy = true;
    error = "";
    try {
      await api<void>(`/entries/${entry.id}`, session.token, {
        method: "DELETE",
      });
      await loadData();
      notice = "Đã xóa giao dịch.";
    } catch (caught) {
      error =
        caught instanceof Error ? caught.message : "Không thể xóa giao dịch.";
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window
  on:keydown={(event) => {
    if (event.key === "Escape" && showForm && !busy) showForm = false;
  }}
/>

{#if loading && !session}
  <main class="loading-screen">
    <span class="spinner"></span>
    <p>Đang mở Driver Notes…</p>
  </main>
{:else if !session}
  <main class="auth-page">
    <div class="auth-layout">
      <section class="auth-story" aria-labelledby="welcome-title">
        <div class="brand brand-light">
          <span class="brand-symbol"><Icon name="truck" size={23} /></span><span
            >driver<span class="brand-strong">notes</span><small
              >NHẬT KÝ TÀI XẾ</small
            ></span
          >
        </div>
        <div class="story-content">
          <p class="story-kicker">MỖI CHUYẾN ĐI ĐỀU ĐÁNG GIÁ</p>
          <h1 id="welcome-title">
            Lái xe tập trung.<br /><em>Thu nhập rõ ràng.</em>
          </h1>
          <p>
            Ghi lại từng đơn hàng, khoản thưởng và chi phí. Biết chính xác hôm
            nay bạn đã làm được bao nhiêu.
          </p>
          <div class="story-line">
            <span></span><span>Đơn giản cho mỗi ngày trên đường</span>
          </div>
        </div>
        <div class="road-art" aria-hidden="true">
          <span class="road road-one"></span><span class="road road-two"
          ></span><span class="road road-three"></span>
        </div>
      </section>
      <section class="auth-panel" aria-labelledby="auth-title">
        <div class="mobile-brand brand">
          <span class="brand-symbol"><Icon name="truck" size={22} /></span><span
            >driver<span class="brand-strong">notes</span></span
          >
        </div>
        <div class="auth-card">
          <span class="eyebrow">BẮT ĐẦU HÀNH TRÌNH</span>
          <h2 id="auth-title">
            {authMode === "login" ? "Chào mừng trở lại" : "Tạo tài khoản"}
          </h2>
          <p class="auth-subtitle">
            {authMode === "login"
              ? "Đăng nhập để xem nhật ký và tiếp tục ca làm của bạn."
              : "Bắt đầu ghi chép thu nhập và chi phí của bạn."}
          </p>
          <form on:submit|preventDefault={authenticate}>
            <label for="email">Địa chỉ email</label><input
              id="email"
              type="email"
              autocomplete="email"
              bind:value={email}
              placeholder="ban@example.com"
              required
            /><label for="password">Mật khẩu</label><input
              id="password"
              type="password"
              autocomplete={authMode === "login"
                ? "current-password"
                : "new-password"}
              bind:value={password}
              placeholder={authMode === "login"
                ? "Nhập mật khẩu"
                : "Ít nhất 12 ký tự"}
              required
              minlength={authMode === "register" ? 12 : undefined}
            />{#if authError || error}<p class="field-error" role="alert">
                {authError || error}
              </p>{/if}<button
              class="button button-primary auth-submit"
              type="submit"
              disabled={busy}
              >{busy
                ? "Đang xử lý…"
                : authMode === "login"
                  ? "Đăng nhập"
                  : "Tạo tài khoản"}
              <Icon name="arrow" size={19} /></button
            >
          </form>
          <p class="auth-switch">
            {authMode === "login" ? "Chưa có tài khoản?" : "Đã có tài khoản?"}
            <button
              type="button"
              on:click={() => {
                authMode = authMode === "login" ? "register" : "login";
                authError = "";
                error = "";
              }}>{authMode === "login" ? "Đăng ký ngay" : "Đăng nhập"}</button
            >
          </p>
        </div>
        <p class="auth-foot">Dữ liệu của bạn được lưu theo tài khoản riêng.</p>
      </section>
    </div>
  </main>
{:else}
  <div class="app-layout" inert={showForm}>
    <aside class="sidebar" aria-label="Điều hướng chính">
      <div class="brand">
        <span class="brand-symbol"><Icon name="truck" size={22} /></span><span
          >driver<span class="brand-strong">notes</span><small
            >NHẬT KÝ TÀI XẾ</small
          ></span
        >
      </div>
      <p class="sidebar-label">KHÔNG GIAN LÀM VIỆC</p>
      <nav class="side-nav" aria-label="Các màn hình">
        {#each nav as item}<button
            class:active={activeTab === item.id}
            aria-current={activeTab === item.id ? "page" : undefined}
            on:click={() => {
              activeTab = item.id;
              notice = "";
              error = "";
            }}
            ><Icon name={item.icon} size={20} /><span>{item.label}</span
            ></button
          >{/each}
      </nav>
      <div class="sidebar-bottom">
        <div class="sidebar-shift">
          <span class:online={!!activeShift} class="status-dot"></span><span
            >{activeShift ? "Đang trong ca" : "Chưa bắt đầu ca"}</span
          >
        </div>
        <div class="sidebar-user">
          <span class="avatar">{session.email.slice(0, 1).toUpperCase()}</span
          ><span class="user-email">{session.email}</span>
        </div>
      </div>
    </aside>
    <div class="main-column">
      <header class="topbar">
        <div class="mobile-brand brand">
          <span class="brand-symbol"><Icon name="truck" size={21} /></span><span
            >driver<span class="brand-strong">notes</span></span
          >
        </div>
        <div class="topbar-date">
          <Icon name="calendar" size={17} /><span>{dayLabel(now)}</span>
        </div>
        <button
          class="icon-button refresh-button"
          aria-label="Tải lại dữ liệu"
          title="Tải lại dữ liệu"
          on:click={() => {
            loading = true;
            void loadData();
          }}
          disabled={loading || busy}><Icon name="refresh" size={19} /></button
        >
      </header>
      <main id="main-content" class="content">
        {#if offline}<div class="banner banner-warning" role="status">
            <Icon name="wifi" size={19} /><span
              >Đang xem dữ liệu đã lưu{snapshotAt
                ? ` · cập nhật ${dateTime(snapshotAt)}`
                : ""}. Cần kết nối mạng để ghi dữ liệu.</span
            >
          </div>{/if}
        {#if error}<div class="banner banner-error" role="alert">
            {error}
          </div>{/if}
        {#if notice}<div class="banner banner-success" role="status">
            <Icon name="check" size={18} />{notice}
          </div>{/if}
        {#if activeTab === "home"}
          <div class="page-heading">
            <div>
              <p class="eyebrow">BẢNG ĐIỀU KHIỂN</p>
              <h1>Hôm nay của bạn</h1>
              <p class="page-subtitle">
                Một cái nhìn rõ ràng cho mỗi ngày làm việc.
              </p>
            </div>
            <span class="today-pill">{dayLabel(now)}</span>
          </div>
          <section class="overview-grid" aria-label="Tổng quan hôm nay">
            <article class="income-card">
              <div class="card-top">
                <span>THU NHẬP HÔM NAY</span><span class="card-icon"
                  ><Icon name="wallet" size={22} /></span
                >
              </div>
              <strong>{vnd(todayIncome)}</strong>
              <div class="income-divider"></div>
              <div class="income-foot">
                <span>Thực nhận sau chi phí</span><b
                  >{vnd(todayIncome - todayExpense)}</b
                >
              </div>
              <div class="card-orbit" aria-hidden="true"></div>
            </article>
            <div class="small-stats">
              <article class="stat-card">
                <span class="stat-icon stat-icon-blue"
                  ><Icon name="truck" size={21} /></span
                ><span class="stat-label">Đơn đã giao</span><strong
                  >{todayOrders}</strong
                ><small>đơn hàng hôm nay</small>
              </article>
              <article class="stat-card">
                <span class="stat-icon stat-icon-orange"
                  ><Icon name="fuel" size={21} /></span
                ><span class="stat-label">Chi phí</span><strong
                  >{vnd(todayExpense)}</strong
                ><small>đã ghi trong ngày</small>
              </article>
            </div>
          </section>
          <div class="home-columns">
            <div class="home-primary">
              <section
                class="section-card shift-card"
                aria-labelledby="shift-card-title"
              >
                <div class="section-header">
                  <div>
                    <p class="eyebrow">CA LÀM HIỆN TẠI</p>
                    <h2 id="shift-card-title">
                      {activeShift
                        ? "Bạn đang trong ca"
                        : "Sẵn sàng lên đường?"}
                    </h2>
                  </div>
                  <span class:running={!!activeShift} class="shift-badge"
                    ><span class="status-dot"></span>{activeShift
                      ? "Đang chạy"
                      : "Chưa bắt đầu"}</span
                  >
                </div>
                <p class="shift-description">
                  {activeShift
                    ? `Bắt đầu lúc ${clock(activeShift.startedAt)} · ${duration(activeShift.startedAt, now)}`
                    : "Bắt đầu ca để theo dõi thời gian làm việc của bạn."}
                </p>
                <button
                  class:button-outline={!!activeShift}
                  class:button-primary={!activeShift}
                  class="button shift-action"
                  on:click={changeShift}
                  disabled={busy || offline}
                  >{activeShift ? "Kết thúc ca" : "Bắt đầu ca làm"}
                  <Icon name="arrow" size={18} /></button
                >
              </section>
              <section
                class="section-card activity-card"
                aria-labelledby="activity-title"
              >
                <div class="section-header">
                  <div>
                    <p class="eyebrow">HOẠT ĐỘNG GẦN ĐÂY</p>
                    <h2 id="activity-title">Giao dịch mới nhất</h2>
                  </div>
                  <button
                    class="text-link"
                    on:click={() => (activeTab = "history")}
                    >Xem tất cả <Icon name="chevron" size={17} /></button
                  >
                </div>
                {#if entries.length === 0}<div class="empty-state">
                    <span class="empty-icon"
                      ><Icon name="list" size={27} /></span
                    ><strong>Chưa có giao dịch nào</strong>
                    <p>Ghi đơn hàng hoặc chi phí đầu tiên để bắt đầu.</p>
                  </div>{:else}<div class="entry-list">
                    {#each entries.slice(0, 4) as entry}<div class="entry-row">
                        <span
                          class:expense={entry.kind === "expense"}
                          class:bonus={entry.kind === "bonus"}
                          class="entry-icon"
                          ><Icon name={entryIcon(entry)} size={20} /></span
                        >
                        <div class="entry-details">
                          <strong>{entryTitle(entry)}</strong><small
                            >{dateTime(entry.occurredAt)}{entry.note
                              ? ` · ${entry.note}`
                              : ""}</small
                          >
                        </div>
                        <span
                          class:negative={entry.kind === "expense"}
                          class="entry-amount"
                          >{entry.kind === "expense" ? "−" : "+"}{vnd(
                            entry.amountVnd +
                              (entry.kind === "delivery" ? entry.tipVnd : 0),
                          )}</span
                        >
                      </div>{/each}
                  </div>{/if}
              </section>
            </div>
            <div class="home-secondary">
              <section
                class="section-card quick-card"
                aria-labelledby="quick-title"
              >
                <p class="eyebrow">GHI NHANH</p>
                <h2 id="quick-title">Thêm một mục mới</h2>
                <p>Vài giây để mọi con số luôn rõ ràng.</p>
                {#each [{ id: "delivery", label: "Đơn giao hàng", icon: "truck" }, { id: "bonus", label: "Tiền thưởng", icon: "gift" }, { id: "expense", label: "Chi phí", icon: "fuel" }] as item}<button
                    class="quick-button"
                    on:click={() => openForm(undefined, item.id as EntryKind)}
                    disabled={offline}
                    ><span><Icon name={item.icon} size={20} /></span
                    >{item.label}<Icon name="chevron" size={18} /></button
                  >{/each}
              </section>
              <section
                class="section-card week-card"
                aria-labelledby="week-title"
              >
                <p class="eyebrow">XU HƯỚNG 7 NGÀY</p>
                <h2 id="week-title">Thu nhập ròng</h2>
                <div
                  class="week-chart"
                  role="img"
                  aria-label={`Thu nhập ròng 7 ngày: ${week.map((d) => `${d.label} ${vnd(d.value)}`).join(", ")}`}
                >
                  {#each week as day}<div class="week-column">
                      <div class="bar-track">
                        <span
                          class:current={day.current}
                          style={`height: ${Math.max(day.value > 0 ? 8 : 0, (Math.max(0, day.value) / weekMax) * 100)}%`}
                        ></span>
                      </div>
                      <small>{day.label}</small>
                    </div>{/each}
                </div>
                <p class="chart-note">Tổng thu nhập trừ chi phí mỗi ngày</p>
              </section>
            </div>
          </div>
        {:else if activeTab === "history"}
          <div class="page-heading">
            <div>
              <p class="eyebrow">NHẬT KÝ TÀI CHÍNH</p>
              <h1>Giao dịch</h1>
              <p class="page-subtitle">
                Mọi khoản thu và chi, theo thứ tự mới nhất.
              </p>
            </div>
            <button
              class="button button-primary heading-action"
              on:click={() => openForm()}
              disabled={offline}
              ><Icon name="plus" size={19} /> Thêm giao dịch</button
            >
          </div>
          <div class="filter-bar" aria-label="Lọc giao dịch">
            {#each [{ id: "all", label: "Tất cả" }, { id: "delivery", label: "Giao hàng" }, { id: "bonus", label: "Thưởng" }, { id: "expense", label: "Chi phí" }] as filter}<button
                class:active={historyFilter === filter.id}
                aria-pressed={historyFilter === filter.id}
                on:click={() =>
                  (historyFilter = filter.id as typeof historyFilter)}
                >{filter.label}</button
              >{/each}
          </div>
          <section
            class="section-card history-card"
            aria-label="Danh sách giao dịch"
          >
            {#if filteredEntries.length === 0}<div class="empty-state">
                <span class="empty-icon"><Icon name="list" size={27} /></span
                ><strong>Chưa có giao dịch</strong>
                <p>
                  {historyFilter === "all"
                    ? "Thêm giao dịch đầu tiên để bắt đầu nhật ký."
                    : "Chưa có giao dịch trong nhóm này."}
                </p>
              </div>{:else}<div class="entry-list">
                {#each filteredEntries as entry}<div
                    class="entry-row history-row"
                  >
                    <span
                      class:expense={entry.kind === "expense"}
                      class:bonus={entry.kind === "bonus"}
                      class="entry-icon"
                      ><Icon name={entryIcon(entry)} size={20} /></span
                    >
                    <div class="entry-details">
                      <strong>{entryTitle(entry)}</strong><small
                        >{dateTime(entry.occurredAt)}{entry.note
                          ? ` · ${entry.note}`
                          : ""}</small
                      >
                    </div>
                    <span
                      class:negative={entry.kind === "expense"}
                      class="entry-amount"
                      >{entry.kind === "expense" ? "−" : "+"}{vnd(
                        entry.amountVnd +
                          (entry.kind === "delivery" ? entry.tipVnd : 0),
                      )}</span
                    >
                    <div class="row-actions">
                      <button
                        class="icon-button"
                        title="Sửa giao dịch"
                        aria-label={`Sửa ${entryTitle(entry)}`}
                        on:click={() => openForm(entry)}
                        disabled={offline || busy}
                        ><Icon name="pencil" size={17} /></button
                      ><button
                        class="icon-button danger"
                        title="Xóa giao dịch"
                        aria-label={`Xóa ${entryTitle(entry)}`}
                        on:click={() => deleteEntry(entry)}
                        disabled={offline || busy}
                        ><Icon name="trash" size={17} /></button
                      >
                    </div>
                  </div>{/each}
              </div>{/if}
          </section>
        {:else if activeTab === "shifts"}
          <div class="page-heading">
            <div>
              <p class="eyebrow">THỜI GIAN LÀM VIỆC</p>
              <h1>Ca làm</h1>
              <p class="page-subtitle">
                Theo dõi hành trình của bạn qua từng ca.
              </p>
            </div>
          </div>
          <section class="section-card shift-page-card">
            <div>
              <span class:running={!!activeShift} class="shift-badge"
                ><span class="status-dot"></span>{activeShift
                  ? "Đang trong ca"
                  : "Chưa có ca đang chạy"}</span
              >
              <h2>
                {activeShift
                  ? `Bắt đầu lúc ${clock(activeShift.startedAt)}`
                  : "Bắt đầu ca mới"}
              </h2>
              <p>
                {activeShift
                  ? `Bạn đã làm việc ${duration(activeShift.startedAt, now)}.`
                  : "Theo dõi thời gian làm việc ngay khi lên đường."}
              </p>
            </div>
            <button
              class:button-outline={!!activeShift}
              class:button-primary={!activeShift}
              class="button"
              on:click={changeShift}
              disabled={offline || busy}
              >{activeShift ? "Kết thúc ca" : "Bắt đầu ca"}
              <Icon name="arrow" size={18} /></button
            >
          </section>
          <section
            class="section-card shift-history"
            aria-labelledby="shift-history-title"
          >
            <div class="section-header">
              <div>
                <p class="eyebrow">LỊCH SỬ CA LÀM</p>
                <h2 id="shift-history-title">Các ca gần đây</h2>
              </div>
            </div>
            {#if shifts.length === 0}<div class="empty-state">
                <span class="empty-icon"><Icon name="clock" size={27} /></span
                ><strong>Chưa có ca làm nào</strong>
                <p>Bắt đầu ca đầu tiên để ghi lại thời gian làm việc.</p>
              </div>{:else}<div class="shift-list">
                {#each shifts as shift}<div class="shift-row">
                    <span class="shift-row-icon"
                      ><Icon name="clock" size={20} /></span
                    >
                    <div>
                      <strong
                        >{new Intl.DateTimeFormat("vi-VN", {
                          weekday: "long",
                          day: "numeric",
                          month: "numeric",
                          year: "numeric",
                        }).format(shift.startedAt)}</strong
                      ><small
                        >{clock(shift.startedAt)} – {shift.endedAt
                          ? clock(shift.endedAt)
                          : "Đang chạy"}</small
                      >
                    </div>
                    <span class="shift-duration"
                      >{duration(shift.startedAt, shift.endedAt ?? now)}</span
                    >
                  </div>{/each}
              </div>{/if}
          </section>
        {:else}
          <div class="page-heading">
            <div>
              <p class="eyebrow">CÀI ĐẶT</p>
              <h1>Tài khoản</h1>
              <p class="page-subtitle">
                Thông tin đăng nhập và trạng thái dữ liệu.
              </p>
            </div>
          </div>
          <section class="section-card account-card">
            <span class="account-avatar"
              >{session.email.slice(0, 1).toUpperCase()}</span
            >
            <div>
              <p class="eyebrow">TÀI KHOẢN CỦA BẠN</p>
              <h2>{session.email}</h2>
              <p>Dữ liệu ca làm và giao dịch gắn với tài khoản này.</p>
            </div>
          </section>
          <section class="section-card data-card">
            <div class="section-header">
              <div>
                <p class="eyebrow">DỮ LIỆU</p>
                <h2>Trạng thái đồng bộ</h2>
              </div>
              <span class:running={!offline} class="shift-badge"
                ><span class="status-dot"></span>{offline
                  ? "Ngoại tuyến"
                  : "Đã kết nối"}</span
              >
            </div>
            <p>
              {snapshotAt
                ? `Lần tải thành công gần nhất: ${dateTime(snapshotAt)}.`
                : "Chưa tải dữ liệu từ máy chủ."}
            </p>
            <button
              class="button button-outline"
              on:click={() => {
                loading = true;
                void loadData();
              }}
              disabled={busy || loading}
              ><Icon name="refresh" size={18} /> Tải lại dữ liệu</button
            >
          </section>
          <button class="button button-logout" on:click={logout} disabled={busy}
            ><Icon name="logout" size={19} /> Đăng xuất</button
          >
        {/if}
      </main>
    </div>
    <nav class="bottom-nav" aria-label="Điều hướng chính trên điện thoại">
      {#each nav as item}<button
          class:active={activeTab === item.id}
          aria-current={activeTab === item.id ? "page" : undefined}
          on:click={() => {
            activeTab = item.id;
            notice = "";
            error = "";
          }}
          ><Icon name={item.icon} size={21} /><span>{item.label}</span></button
        >{/each}
    </nav>
  </div>
{/if}

{#if showForm}
  <div class="modal-backdrop">
    <div
      class="entry-modal"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="entry-form-title"
    >
      <header class="modal-header">
        <div>
          <p class="eyebrow">NHẬT KÝ TÀI XẾ</p>
          <h2 id="entry-form-title">
            {editingId ? "Sửa giao dịch" : "Thêm giao dịch"}
          </h2>
        </div>
        <button
          class="icon-button"
          aria-label="Đóng biểu mẫu"
          on:click={() => (showForm = false)}
          disabled={busy}><Icon name="x" size={21} /></button
        >
      </header>
      <form on:submit|preventDefault={saveEntry} class="entry-form">
        <fieldset class="kind-picker">
          <legend>Loại giao dịch</legend>
          <div>
            {#each [{ id: "delivery", label: "Giao hàng", icon: "truck" }, { id: "bonus", label: "Thưởng", icon: "gift" }, { id: "expense", label: "Chi phí", icon: "fuel" }] as option}<button
                type="button"
                class:active={kind === option.id}
                aria-pressed={kind === option.id}
                on:click={() => (kind = option.id as EntryKind)}
                ><Icon name={option.icon} size={19} />{option.label}</button
              >{/each}
          </div>
        </fieldset>
        <div class="form-grid">
          <div class="form-field">
            <label for="entry-amount">Số tiền (VND) <span>*</span></label><input
              id="entry-amount"
              type="number"
              inputmode="numeric"
              min="1"
              max="1000000000000"
              step="1"
              bind:value={amount}
              placeholder="Ví dụ: 50000"
              required
            />
          </div>
          {#if kind === "delivery"}<div class="form-field">
              <label for="entry-tip">Tiền tip (VND)</label><input
                id="entry-tip"
                type="number"
                inputmode="numeric"
                min="0"
                max="1000000000000"
                step="1"
                bind:value={tip}
                placeholder="0"
              />
            </div>
            <div class="form-field">
              <label for="entry-count">Số đơn trong lượt giao</label><select
                id="entry-count"
                bind:value={deliveryCount}
                ><option value={1}>1 đơn</option><option value={2}>2 đơn</option
                ><option value={3}>3 đơn</option></select
              >
            </div>{/if}{#if kind === "expense"}<div class="form-field">
              <label for="entry-category">Loại chi phí</label><select
                id="entry-category"
                bind:value={expenseCategory}
                ><option value="fuel">Xăng xe</option><option value="food"
                  >Ăn uống</option
                ><option value="other">Khác</option></select
              >
            </div>{/if}
          <div class="form-field">
            <label for="entry-date">Thời gian phát sinh</label><input
              id="entry-date"
              type="datetime-local"
              bind:value={occurredAt}
              required
            />
          </div>
          <div class="form-field form-wide">
            <label for="entry-shift">Gắn với ca làm</label><select
              id="entry-shift"
              bind:value={selectedShiftId}
              ><option value="">Không gắn với ca</option
              >{#each shifts as shift}<option value={shift.id}
                  >{dateTime(shift.startedAt)}{shift.endedAt
                    ? ""
                    : " · đang chạy"}</option
                >{/each}</select
            >
          </div>
          <div class="form-field form-wide">
            <label for="entry-note">Ghi chú</label><textarea
              id="entry-note"
              rows="3"
              maxlength="500"
              bind:value={note}
              placeholder="Thêm chi tiết nếu cần…"
            ></textarea>
          </div>
        </div>
        {#if formError}<p class="field-error" role="alert">{formError}</p>{/if}
        <div class="modal-actions">
          <button
            type="button"
            class="button button-plain"
            on:click={() => (showForm = false)}
            disabled={busy}>Hủy</button
          ><button type="submit" class="button button-primary" disabled={busy}
            >{busy ? "Đang lưu…" : editingId ? "Lưu thay đổi" : "Lưu giao dịch"}
            <Icon name="arrow" size={18} /></button
          >
        </div>
      </form>
    </div>
  </div>
{/if}
