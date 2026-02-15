let rawGroups = [], rawLeagues = [], rawTeams = [];

const groupSelect = document.getElementById('groupSelect');
const leagueSelect = document.getElementById('leagueSelect');
const teamSelect = document.getElementById('teamSelect');
const calendarUrl = document.getElementById('calendarUrl');

function normalizeLeagueGroup(group) {
  return {
    id: group.mundial_league_group_id,
    name: group.name
  };
}

function normalizeLeague(league) {
  return {
    id: league.mundial_league_id,
    group_id: league.mundial_league_group_id,
    name: league.name
  };
}

function normalizeTeam(team) {
  return {
    id: team.mundial_team_id,
    league_id: team.mundial_league_id,
    name: team.name
  };
}

async function loadData() {
  [rawGroups, rawLeagues, rawTeams] = await Promise.all([
    fetch('/api/mundial_league_groups.json').then(res => res.json()),
    fetch('/api/mundial_leagues.json').then(res => res.json()),
    fetch('/api/mundial_teams.json').then(res => res.json())
  ]);

  const groups = rawGroups.map(normalizeLeagueGroup);
  populateSelect(groupSelect, groups);
}

function populateSelect(select, items, valueField = 'id', labelField = 'name') {
  select.innerHTML = '<option value="">-- Select --</option>';
  items.forEach(item => {
    const option = document.createElement('option');
    option.value = item[valueField];
    option.textContent = item[labelField];
    select.appendChild(option);
  });
  select.disabled = false;
}

groupSelect.addEventListener('change', () => {
  const selectedGroup = parseInt(groupSelect.value);
  const leagues = rawLeagues.map(normalizeLeague).filter(l => l.group_id === selectedGroup);
  populateSelect(leagueSelect, leagues);
  teamSelect.innerHTML = '<option value="">-- Select --</option>';
  teamSelect.disabled = true;
  calendarUrl.style.display = 'none';
});

leagueSelect.addEventListener('change', () => {
  const selectedLeague = parseInt(leagueSelect.value);
  const teams = rawTeams.map(normalizeTeam).filter(t => t.league_id === selectedLeague);
  populateSelect(teamSelect, teams);
  calendarUrl.style.display = 'none';
});

teamSelect.addEventListener('change', () => {
  const selectedTeamId = teamSelect.value;
  const selectedTeam = rawTeams.map(normalizeTeam).find(t => t.id == selectedTeamId);

  if (selectedTeam) {
    const url = `https://d39amfcda6iyyg.cloudfront.net/football_mundial/${selectedTeam.id}.ics`;
    calendarUrl.href = url;
    calendarUrl.textContent = `Copy Calendar Link`;
    calendarUrl.style.display = 'block';
  }
});

calendarUrl.addEventListener('click', (e) => {
  e.preventDefault();
  navigator.clipboard.writeText(calendarUrl.href).then(() => {
    calendarUrl.textContent = "Copied!";
    setTimeout(() => {
      const selectedTeamId = teamSelect.value;
      const selectedTeam = rawTeams.map(normalizeTeam).find(t => t.id == selectedTeamId);
      if (selectedTeam) {
        calendarUrl.textContent = `Copy ${selectedTeam.name} Calendar Link`;
      }
    }, 1500);
  });
});

loadData();

