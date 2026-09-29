# 02 — antonpup/Dota2GSI (C#): Análise da Implementação

**Repo:** https://github.com/antonpup/Dota2GSI | **Release:** v2.1.1 (02/2024) | **Stack:** C# / .NET 8 / Newtonsoft.Json 13.0.3 | **NuGet:** `Dota2GSI` (~11.8k downloads) | **Build:** CMake 3.26+ → `build/Dota2GSI.sln`, CI só Windows

## 1. Propósito

Consumir GSI sem memória/anti-cheat para overlays, RGB, plugins de stream e estatística live. Herdada da lib CSGO-GSI (`rakijah`).

## 2. Arquitetura

```
Dota --POST--> HttpListener --JObject.Parse--> GameState
  --> NewGameState (snapshot) + GameStateHandler (diff por Equals)
    --> 17 StateHandlers (Abilities, Hero, Items, Map, Buildings, Roshan, Draft, etc.)
      --> ~90 eventos finos + GameEvent genérico via EventDispatcher
```

- `GameStateListener`: fachada (`Port/URI/Running`, `Start()/Stop()/Dispose()`, `GenerateGSIConfigFile(name)`). `new GameStateListener(3000)` ou URI com `/` final. Fora de `localhost` exige admin (reserva HttpListener).
- `Node / NodeMap / NodeList`: base de parsing com `GetString/GetInt/GetLong/GetFloat/GetBool/GetEnum` + defaults (`bool→false, int→-1, string→Empty, enum→Undefined`). `Equals` por `JToken.Equals` permite diff.
- `EventDispatcher<T>`: `Subscribe/Unsubscribe/Broadcast`, propaga para `GameEvent`.
- `Dota2EventsInterface`: ~90 `delegate+event` (`TimeOfDayChanged`, `InventoryItemAdded`, `TowerDestroyed`...).
- `Dota2GSIFile + SteamUtils + ACF`: gera `.cfg` achando o Dota via registro `HKLM\SOFTWARE\Valve\Steam` + `libraryfolders.vdf` + `appmanifest_570.acf`.

## 3. GameState (16 nós)

`Auth, Provider {Name, AppID 570, Version, TimeStamp}, Map {MatchID, GameTime, ClockTime, IsDaytime, IsNightstalkerNight, RadiantScore/DireScore, GameState (12 valores), IsPaused, WinningTeam, RoshanState}, Player {LocalPlayer + Teams}, Hero {HP/Mana/XP/Level/Talents/SelectedUnit...}, Abilities, Items {slot/stash/teleport/neutral}, Events (só 6 tipos: Courier_killed, Roshan_killed, Aegis_*, Tip, Bounty_rune), Buildings, League, Draft (torneio), Wearables, Minimap, Roshan (spectator), Couriers, NeutralItems` + `Previously` + derivados `LocalPlayer, Radiant/DireTeamDetails, IsSpectating, IsLocalPlayer`.

## 4. Exemplo de uso documentado no repo

```csharp
var gsl = new GameStateListener(3000);
gsl.GenerateGSIConfigFile("MeuColetor");
gsl.NewGameState += gs => Console.WriteLine($"{gs.Map.ClockTime}s {gs.Map.RadiantScore}x{gs.Map.DireScore}");
gsl.InventoryItemAdded += e => Console.WriteLine($"{e.Player.Details.Name} ganhou {e.Value.Name}");
gsl.Start();
```

## 5. Limitações e riscos observados

- Windows-only na prática (`HttpListener` + registro). Porta em uso/sem permissão = `Start()` retorna `false`.
- Bug observado: `HeroState` usa `&=` em vez de `|=` → sempre `None`.
- Sem validação de `auth.token`; resposta sempre 200 mesmo com JSON inválido; `DynamicInvoke` sob lock (lento).
- Licença ambígua: `LICENSE.md` contém só texto MIT do Json.NET. Sem declaração própria do autor.
