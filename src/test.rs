use super::*;

#[test]
fn game_over_top_nine_away_winning() {
    let state = GameState {
        half_inning: HalfInning::Top,
        inning: 9,
        home_team_runs: 0,
        away_team_runs: 1,
    };

    assert!(!state.is_over());
}

#[test]
fn game_over_top_nine_home_winning() {
    let state = GameState {
        half_inning: HalfInning::Top,
        inning: 9,
        home_team_runs: 1,
        away_team_runs: 0,
    };

    assert!(state.is_over());
}

#[test]
fn game_over_top_nine_tied() {
    let state = GameState {
        half_inning: HalfInning::Top,
        inning: 9,
        home_team_runs: 0,
        away_team_runs: 0,
    };

    assert!(!state.is_over());
}

#[test]
fn game_over_bottom_nine_away_winning() {
    let state = GameState {
        half_inning: HalfInning::Bottom,
        inning: 9,
        home_team_runs: 0,
        away_team_runs: 1,
    };

    assert!(state.is_over());
}

#[test]
fn game_over_bottom_nine_home_winning() {
    let state = GameState {
        half_inning: HalfInning::Bottom,
        inning: 9,
        home_team_runs: 1,
        away_team_runs: 0,
    };

    assert!(state.is_over());
}

#[test]
fn game_over_bottom_nine_tied() {
    let state = GameState {
        half_inning: HalfInning::Bottom,
        inning: 9,
        home_team_runs: 0,
        away_team_runs: 0,
    };

    assert!(!state.is_over());
}

#[test]
fn game_over_top_away_winning() {
    let state = GameState {
        half_inning: HalfInning::Top,
        inning: 10,
        home_team_runs: 0,
        away_team_runs: 1,
    };

    assert!(!state.is_over());
}

#[test]
fn test_inning_changes() {
    let mut state = GameState::default();

    assert_eq!(state.inning, 0);
    assert_eq!(state.half_inning, HalfInning::Bottom);

    state.step(0);

    assert_eq!(state.inning, 1);
    assert_eq!(state.half_inning, HalfInning::Top);

    state.step(0);

    assert_eq!(state.inning, 1);
    assert_eq!(state.half_inning, HalfInning::Bottom);

    state.step(0);

    assert_eq!(state.inning, 2);
    assert_eq!(state.half_inning, HalfInning::Top);
}
