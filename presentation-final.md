---
title: Agendamento de Pousos de Aeronaves - O Caso Estático
description: Uma exploração do problema de agendamento de pousos de aeronaves, incluindo restrições, objetivos e aplicações práticas.
# theme: frankfurt
infoLine: false
author: 'Eduardo Henke' # shows in infoLine
colorSchema: light

---

# Agendamento de Pousos de Aeronaves com Simulated Annealing

Eduardo Henke

---

# Contexto do Problema

- Ao entrar no alcance do radar do controle de tráfego aéreo (ATC): o avião precisa ser atribuído um **horário de pouso** dentro de uma janela de tempo específica.
- **Objetivo:** Garantir pousos seguros mantendo a separação entre aeronaves e minimizando os custos relacionados ao desvio do horário de pouso preferido.

---

# Restrições

- **Janela de Tempo:**
  - Cada avião $i$ tem um horário de pouso mais cedo $E_i$ e um mais tarde $L_i$.
    - $E_i$: Horário mais cedo se o avião voar à velocidade máxima.
    - $L_i$: Horário mais tarde, considerando a eficiência de combustível e o tempo máximo de espera.
    - $T_i$: Horário de pouso preferido.
- **Tempo de Separação:**
  - Cada par de aviões $(i, j)$ deve manter um tempo mínimo de separação $S_{ij}$ entre seus pousos:
    - Exemplo: Um Boeing 747 precisa de mais tempo de separação em relação a um avião menor, por conta da turbulência gerada.

<!-- **Gráfico/Diagrama:** Gráfico de uma linha do tempo mostrando as janelas de tempo de pouso e as separações entre os aviões.  
Adicione aqui um gráfico de linha do tempo mostrando as janelas de pouso e o tempo de separação -->

---

# Função Objetivo - Minimizar o Custo Total

- O custo é incorrido quando o avião pousa antes ou depois do horário alvo $T_i$:
  - Pouso antes de $T_i$ gera um custo, pouso depois também.
  - O objetivo é minimizar a soma dos custos para todos os aviões.

![Diagrama de Custo por Pousos](./img/cost.png)

---

# Exemplo

**Objetivo:** Determinar os horários de pouso $x_i$ para cada avião $i$ de forma que:
1. $x_i \in [E_i, L_i]$, ou seja, o avião pouse dentro da janela de tempo.
2. A condição de separação $x_j \geq x_i + S_{ij}$ seja respeitada para cada par de aviões $(i, j)$ subsequentes.

- **Exemplo Simples:**
  - O avião A tem uma janela de tempo de 10h às 10h30.
  - O avião B tem uma janela de 10h20 às 11h.
  - Se o tempo de separação for 10 minutos, o avião A deve pousar antes de B, com uma separação mínima de 10 minutos.
  - Exemplo: `[(A, 10h05), (B, 10h20)]`; `[(B, 10h20), (A, 10h30)]`

---

# Representação da solução

- Representação da solução, é uma lista ordenada de pousos, onde um pouso é uma tupla `(ID Avião, Horário de Pouso)`.
- A estrutura de vizinhança então será alterar o horário de um pouso e reordenar a lista baseado nesse novo horário.

---

# Estratégias de Busca

- **Randômica:** selecionar um pouso aleatório e alterar o seu horário aleatoriamente dentro da janela de tempo, não respeitando o tempo de separação.

```
procedure random_neighbor(solution) {
  i = random.choice(solution.len())
  solution[i].time = random.uniform(solution[i].earliest, solution[i].latest)
  return solution
}
```

- **First Improvement:** itera aleatoriamente sobre todos os pousos e altera o seu horário de uma forma "zig-zag" dentro da janela de tempo.
  - Pouso $i$ com horário $x_i$, tenta horários $x_i ; x_i-1 ; x_i+1 ; x_i-2 ; x_i+2 ; ... ; E_i ; L_i$.

```
procedure first_improvement_neighbor(solution) {
  for arrival in shuffle(solution) {
    for time in zigzag_range(arrival.earliest, arrival.latest, arrival.time) {
      if changing arrival with time results in a better solution {
        return changed_solution
      }
    }
  }
}
```
---

# Implementação - Loop Principal

- **Solução Construtiva Inicial**: criar uma lista de pousos com os horários $T_i$ de cada avião $\rightarrow$ terá vários conflitos.
- Repetir até chegar no tempo limite:
  - **Simulated Annealing**: aplicar SA na solução construtiva inicial.
  - **Busca First Improvement**: aplicar FI na solução dada pelo SA.

---

# Implementação - Função Objetivo

Permitimos soluções inválidas, porém com penalidade para conflitos.

```python
procedure cost(solution) {
  landing_cost(solution) + conflict_cost(solution)
}

procedure landing_cost(solution) {
  sum(arrival.plane.cost_for_landing(arrival.landing_time) for arrival in solution)
}

procedure conflict_cost(solution) {
  sum(CONFLICT_PENALTY * conflict.duration for conflict in solution.conflicts())
}
```

---

# IRace

<br>

Projeto foi configurado com o IRace para realizar o *tuning* dos parâmetros para o SA, especialmente:

```
sa_max_k      ""  r (0.1, 8.0)
alpha         ""  r (0.8, 1.0)
initial_temp  ""  r (100.0, 1000000.0)
```

<br>

O resultado foi o seguinte:

```
   sa_max_k  alpha initial_temp
45   3.8272 0.9809   10599.1518
24   5.7757 0.9565  219812.3305
```

<br>
<br>

> IRace pode rodar o programa em até $10*n^2$ iterações.

---
layout: image
image: ./img/table.png
backgroundSize: contain
---

---

# Resultados obtidos com >1s de execução

| **#** | **n** | **Optimal** | **Paper** | **Mine**  |
|---|---|---|---|---|
| 1 | 10 | 700 | 700 | 700 |
| 2 | 15 | 1480 | 1500 | 1480 |
| 3 | 20 | 820 | 1380 | 820 |
| 4 | 20 | 2520 | 2520 | 2820 |
| 5 | 20 | 3100 | 5420 | 3950 |
| 6 | 30 | 24442 | 24442 | ~~20054~~* |
| 7 | 44 | 1550 | 1550 | 1588 |
| 8 | 50 | 1950 | 2690 | 2480 |

---

# Extra

Gráficos de melhoria de performance:

```
file:///Users/henke/ufop/metaheuristics/airplane-landing-scheduler/img/flamegraph.svg
```

```
samply record ./target/release/airplane-landing-scheduler eval-one 3.8272 0.9809 10599.1518 data/airland4.txt 2.0
```

```
file:///Users/henke/ufop/metaheuristics/airplane-landing-scheduler/perf-cmp-1/report/index.html
```



---
layout: end
---

Obrigado!